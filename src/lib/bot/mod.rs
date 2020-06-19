pub mod command;
pub mod config;
pub mod util;

use std::collections::HashMap;
use std::iter::FromIterator;

use mlua::{UserData, UserDataMethods};
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel};

use crate::{
    stream_elements::consumer::ConsumerStreamElementsAPI, youtube::ConsumerYouTubePlaylistAPI,
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
                    log::info!("Failed to reload `{}`: {}", _message, e);
                    self.send(&evt.channel, e.to_string()).await;
                }
            }
            return;
        }

        // Not the most clean and fragrant piece code
        // but demonstrates the usage of tokio::spawn.
        if evt.data.starts_with("xD queue") && self.is_boss(&evt.name) {
            let parts: Vec<String> = evt
                .data
                .split_whitespace()
                .skip(2)
                .map(String::from)
                .collect();
            let yt = self.youtube_playlist.clone().unwrap();
            let se = self.streamelements.clone().unwrap();
            tokio::spawn(async move {
                let id = parts.first().unwrap();
                let num = parts.last().unwrap().parse::<usize>().unwrap();
                yt.configure(id.to_owned(), num).await.unwrap();
                let song_urls = yt
                    .get_playlist_videos()
                    .await
                    .unwrap()
                    .into_videos()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.into_url())
                    .collect();
                log::trace!(
                    "{:?}",
                    se.song_requests().queue_many(song_urls).await.unwrap()
                );
            });
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
            let header = vec![evt.channel.to_string(), evt.name.to_string()];
            let args: mlua::Variadic<String> = if let Some(args) = args {
                mlua::Variadic::from_iter(
                    header
                        .into_iter()
                        .chain(args.into_iter().map(|it| it.to_owned())),
                )
            } else {
                mlua::Variadic::from_iter(header.into_iter())
            };
            let response = (command.script)
                .call_async::<mlua::Variadic<String>, String>(args)
                .await;
            let response = match response {
                Ok(resp) if resp.is_empty() => return,
                Ok(response) => response,
                Err(e) => {
                    log::error!("Failed to execute script: {:?}", e);
                    format!("WAYTOODANK devs broke something!")
                }
            };
            self.send(&evt.channel, response).await;
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

pub fn init_api_globals<'lua>(lua: &'lua mlua::Lua, api: APIStorage) {
    if let Err(e) = lua.globals().set("api", api) {
        log::error!("Failed to set global object \"api\": {}", e);
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
