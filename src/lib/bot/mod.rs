pub mod command;
pub mod config;
pub mod util;

use crate::{
    stream_elements::consumer::ConsumerStreamElementsAPI, youtube::ConsumerYouTubePlaylistAPI,
};
use command::{load_commands, Command};
use mlua::{UserData, UserDataMethods};
use std::collections::HashMap;
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel};

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
        let commands: HashMap<String, Command<'lua>> = load_commands(lua, "commands.json");

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
    control: Control, // exposed through Bot::stop
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
        BotInfo { start: self.start }
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
            match &*event {
                messages::AllCommands::Privmsg(msg) => {
                    self.handle_msg(msg, lua).await;
                }
                _ => {}
            }
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
                    self.send(&evt.channel, format!("WAYTOODANK ❗❗ something broke"))
                        .await;
                }
            }
            return;
        }

        // Not the most clean and fragrant piece code
        // but demonstrates the usage of tokio::spawn.

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
            // TODO: make lua state Arc<RwLock<Lua>>
            // so that we can send mlua::Function between threads
            // enabling us to spawn tasks for expensive commands
            /*
            if command.is_expensive {
                // spawn task for command
            } else {
                // execute the command here
            }
            */
            let response = match command
                .script
                .clone()
                .call_async::<mlua::Variadic<String>, Option<String>>(util::format_args(evt, args))
                .await
            {
                Ok(resp) if resp.is_none() => {
                    return;
                }
                Ok(resp) => resp.unwrap(),
                Err(e) => {
                    log::error!("Failed to execute script: {:?}", e);
                    format!("WAYTOODANK devs broke something!")
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
        self.control
            .writer()
            .privmsg(channel, message.into())
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

pub fn init_api_globals<'lua>(lua: &'lua mlua::Lua, api: APIStorage, bot: BotInfo) {
    let globals = lua.globals();
    if let Err(e) = globals.set("api", api) {
        panic!(format!("{}", e));
    }
    if let Err(e) = globals.set("bot", bot) {
        panic!(format!("{}", e));
    }
}

pub struct BotInfo {
    pub start: chrono::DateTime<chrono::Utc>,
}

impl UserData for BotInfo {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("uptime", |_, instance, ()| {
            Ok(util::duration_format(chrono::Utc::now() - instance.start))
        });
    }
}

pub struct APIStorage {
    pub streamelements: Option<ConsumerStreamElementsAPI>,
    pub youtube_playlist: Option<ConsumerYouTubePlaylistAPI>,
}

impl UserData for APIStorage {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("streamelements", |_, instance, ()| {
            Ok(instance.streamelements.clone())
        });
        methods.add_method("youtube_playlist", |_, instance, ()| {
            Ok(instance.youtube_playlist.clone())
        });
    }
}
