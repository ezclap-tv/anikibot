#[macro_use]
pub mod macros;
pub mod command;
pub mod config;
pub mod util;

use std::collections::HashMap;

use mlua::{ToLua, UserData, UserDataMethods};
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel};

use crate::{
    stream_elements::consumer::ConsumerStreamElementsAPI, youtube::ConsumerYouTubePlaylistAPI,
    BackendError,
};
use command::{load_commands, Command};

/* Previously had commands: ping, ping uptime, whoami, song, song queue */

pub struct BotBuilder {
    streamelements_api: Option<ConsumerStreamElementsAPI>,
    youtube_api: Option<ConsumerYouTubePlaylistAPI>,
    control: Control,
}

impl BotBuilder {
    pub fn add_streamelements_api(self, streamelements_api: ConsumerStreamElementsAPI) -> Self {
        BotBuilder {
            streamelements_api: Some(streamelements_api),
            ..self
        }
    }

    pub fn add_youtube_api(self, youtube_api: ConsumerYouTubePlaylistAPI) -> Self {
        BotBuilder {
            youtube_api: Some(youtube_api),
            ..self
        }
    }

    pub fn build<'lua>(self, lua: &'lua mlua::Lua) -> Bot<'lua> {
        let commands: HashMap<String, Command<'lua>> =
            load_commands(lua, "commands.json").expect("Failed to load the commands");

        Bot {
            streamelements: self.streamelements_api,
            youtube_playlist: self.youtube_api,
            control: self.control,
            config: config::BotConfig::get(),
            start: chrono::Utc::now(),
            commands,
        }
    }
}

pub struct Bot<'lua> {
    pub streamelements: Option<ConsumerStreamElementsAPI>,
    pub youtube_playlist: Option<ConsumerYouTubePlaylistAPI>,
    control: Control,
    config: config::BotConfig,
    pub start: chrono::DateTime<chrono::Utc>,
    pub commands: HashMap<String, Command<'lua>>,
}

impl<'lua> Bot<'lua> {
    pub fn builder(control: Control) -> BotBuilder {
        BotBuilder {
            streamelements_api: None,
            youtube_api: None,
            control,
        }
    }

    pub fn get_bot_info(&self) -> BotInfo {
        BotInfo {
            start: self.start,
            control: self.control.clone(),
        }
    }

    pub fn get_api_storage(&self) -> APIStorage {
        APIStorage {
            streamelements: self.streamelements.clone(),
            youtube_playlist: self.youtube_playlist.clone(),
        }
    }

    #[inline]
    pub fn is_boss(&self, name: &str) -> bool {
        self.config.gym_staff.contains(name)
    }

    pub async fn run(mut self, lua: &mlua::Lua, dispatcher: Dispatcher) {
        let mut events = dispatcher.subscribe::<events::All>();

        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        self.join_configured_channels(&ready.nickname).await;

        self.send(
            &"moscowwbish".into_channel().unwrap(),
            "gachiHYPER I'M READY",
        )
        .await;

        while let Some(event) = events.next().await {
            if let messages::AllCommands::Privmsg(msg) = &*event {
                self.handle_msg(msg, lua).await;
            };
        }
    }

    pub fn stop(&mut self) {
        self.control.stop();
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>, lua: &'lua mlua::Lua) {
        if !evt.data.starts_with("xD") {
            return;
        }

        // hardcoded "xD" response because it needs to exist
        if evt.data.trim() == "xD" {
            self.send(&evt.channel, "xD").await;
            return;
        }

        if evt.data.trim() == "xD stop" && self.is_boss(&evt.name) {
            self.stop();
            return;
        }

        if evt.data.starts_with("xD reload all") && self.is_boss(&evt.name) {
            log::info!("Attempting to reload commands.json");
            match load_commands(lua, "commands.json") {
                Ok(commands) => {
                    log::info!("Successfully reloaded commands.json");
                    self.commands = commands;
                    self.send(
                        &evt.channel,
                        format!("👉 Successfully reloaded the commands file."),
                    )
                    .await
                }
                Err(e) => {
                    log::error!("Failed to reload commands.json: {}", e);
                    self.send(&evt.channel, "WAYTOODANK ❗❗ something broke".to_owned())
                        .await;
                }
            }
            return;
        }

        if evt.data.starts_with("xD reload ") && self.is_boss(&evt.name) {
            let _message = util::strip_prefix(&evt.data, "xD reload ");
            match util::reload_command(&mut self.commands, _message, |cmd| {
                util::load_file(&cmd.path)
                    .and_then(|source| command::load_lua(&lua, _message, &source))
                    .map(|script| cmd.script = script)
                    .map_err(|e| e.inner)
            }) {
                Ok(_) => {
                    self.send(
                        &evt.channel,
                        format!("👉 Successfully reloaded `{}`", _message),
                    )
                    .await
                }
                Err(e) => {
                    log::error!("Failed to reload `{}`: {}", _message, e);
                    self.send(&evt.channel, "WAYTOODANK ❗❗ something broke".to_owned())
                        .await;
                }
            }
            return;
        }

        if evt.data.starts_with("xD help ") {
            let name = util::strip_prefix(&evt.data, "xD help ");
            log::info!("Help for command {}", name);
            let (data, _) = match util::find_command(&self.commands, &name) {
                Some(found) => found,
                None => {
                    return;
                }
            };
            self.send(&evt.channel, format!("FeelsDankMan 👉 {}", data.usage))
                .await;
            return;
        }

        let message = util::strip_prefix(&evt.data, "xD ");
        if let Some((command, args)) = util::find_command(&self.commands, message) {
            if command.is_expensive {
                let thread_name = format!(
                    "cmd-{}@{{bot_uptime = {}}}",
                    command.name,
                    util::duration_format(chrono::Utc::now() - self.start),
                );
                log::info!(
                    "Command `{}` is expensive, spawning a new thread named `{}`",
                    command.name,
                    thread_name,
                );
                let path = command.path.clone();
                let local_lua = mlua::Lua::new();
                let args = util::format_args(evt, args);
                let channel = evt.channel.clone().into_owned();
                let mut control = self.control.clone();
                crate::lua::init_globals_for_lua(&local_lua, &self);

                std::thread::Builder::new()
                    .name(thread_name)
                    .spawn(move || {
                        let start = chrono::Utc::now();
                        (|| {
                            let fun = thread_try!(
                                util::load_file(&path).and_then(|source| {
                                    local_lua
                                        .load(&source)
                                        .into_function()
                                        .map_err(|e| BackendError::from(format!("{}", e)))
                                }),
                                "Failed to compile the script: {:?}"
                            );

                            let mut rt = thread_try!(
                                tokio::runtime::Runtime::new(),
                                "Failed to create a new tokio runtime: {:?}"
                            );
                            rt.block_on(async move {
                                let response = fun
                                    .call_async::<mlua::Variadic<String>, Option<String>>(args)
                                    .await;
                                let response = match response {
                                    Ok(Some(resp)) => resp,
                                    Ok(None) => return,
                                    Err(e) => {
                                        thread_error!("Failed to execute script: {:?}", e);
                                        send_in_thread(
                                            &mut control,
                                            &channel,
                                            "WAYTOODANK devs broke something!",
                                        )
                                        .await;
                                        return;
                                    }
                                };
                                send_in_thread(&mut control, &channel, response).await;
                            })
                        })();
                        thread_info!(
                            "Terminating the task thread (thread lived for {})",
                            util::duration_format(chrono::Utc::now() - start)
                        );
                    })
                    .unwrap();
                return;
            }
            let response = match command
                .script
                .clone()
                .call_async::<mlua::Variadic<String>, Option<String>>(util::format_args(evt, args))
                .await
            {
                Ok(Some(resp)) => resp,
                Ok(None) => return,
                Err(e) => {
                    log::error!("Failed to execute script: {:?}", e);
                    "WAYTOODANK devs broke something!".to_owned()
                }
            };
            self.send(&evt.channel.clone(), response).await;
        }
    }

    pub async fn join(&mut self, channel: &str, nickname: &str) {
        log::info!("Connected to {} as {}", &channel, nickname);
        self.control
            .writer()
            .join(channel)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Caught a critical error while joining a channel {}: {:?}",
                    channel,
                    e
                );
            })
    }

    async fn join_configured_channels(&mut self, nickname: &str) {
        for channel in self.config.channels.iter() {
            log::info!("Connected to {} as {}", &channel, nickname);
            self.control
                .writer()
                .join(channel)
                .await
                .unwrap_or_else(|e| {
                    log::error!(
                        "Caught a critical error while joining a channel {}: {:?}",
                        channel,
                        e
                    );
                })
        }
    }

    async fn send<S: Into<String>>(&mut self, channel: &str, message: S) {
        send(&mut self.control, channel, message)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Caught a critical error while sending a response to the channel {}: {:?}",
                    channel,
                    e
                );
            })
    }
}

async fn send<S: Into<String>>(
    control: &mut Control,
    channel: &str,
    message: S,
) -> Result<(), twitchchat::Error> {
    control.writer().privmsg(channel, message.into()).await
}

async fn send_in_thread<S: Into<String>>(control: &mut Control, channel: &str, message: S) {
    send(control, channel, message).await.unwrap_or_else(|e| {
        thread_error!(
            "Caught a critical error while sending a response to the channel {}: {:?}",
            channel,
            e
        );
    })
}

pub fn init_api_globals(lua: &mlua::Lua, api: APIStorage, bot: BotInfo) {
    let globals = lua.globals();
    if let Err(e) = globals.set("api", api) {
        panic!(format!("{}", e));
    }
    if let Err(e) = globals.set("bot", bot) {
        panic!(format!("{}", e));
    }
}

#[derive(Clone)]
pub struct BotInfo {
    pub start: chrono::DateTime<chrono::Utc>,
    control: Control,
}

impl UserData for BotInfo {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("uptime", |_, instance, ()| {
            Ok(util::duration_format(chrono::Utc::now() - instance.start))
        });
        methods.add_async_method(
            "send",
            |lua, mut instance, (chan, msg): (String, String)| async move {
                let res = instance.control.writer().privmsg(&chan, msg).await;
                Ok(match res {
                    Ok(()) => (mlua::Value::Boolean(true), mlua::Value::Nil),
                    Err(e) => (
                        mlua::Value::Nil,
                        mlua::Value::String(lua.create_string(&e.to_string())?),
                    ),
                })
            },
        );
    }
}

pub struct APIStorage {
    pub streamelements: Option<ConsumerStreamElementsAPI>,
    pub youtube_playlist: Option<ConsumerYouTubePlaylistAPI>,
}

impl UserData for APIStorage {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("streamelements", |lua, instance, ()| {
            Ok(match instance.streamelements.clone() {
                Some(api) => (api.to_lua(lua)?, mlua::Value::Nil),
                None => (
                    mlua::Value::Nil,
                    mlua::Value::String(lua.create_string("StreamElements API is unavailable!")?),
                ),
            })
        });
        methods.add_method("youtube_playlist", |lua, instance, ()| {
            Ok(match instance.youtube_playlist.clone() {
                Some(api) => (api.to_lua(lua)?, mlua::Value::Nil),
                None => (
                    mlua::Value::Nil,
                    mlua::Value::String(lua.create_string("YouTube API is unavailable!")?),
                ),
            })
        });
    }
}
