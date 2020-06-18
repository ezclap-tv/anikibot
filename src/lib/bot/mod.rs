pub mod command;
pub mod config;
pub mod util;

use std::collections::HashMap;
use std::iter::FromIterator;

use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel};

use crate::{
    stream_elements::consumer::ConsumerStreamElementsAPI, youtube::ConsumerYouTubePlaylistAPI,
};
use command::{Command, load_commands};

/* Previously had commands: help, ping, ping uptime, whoami, stop, song, song queue*/

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

    pub fn build(self) -> Bot {
        let commands: HashMap<String, Command> = load_commands( "commands.json");
        

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

pub struct Bot {
    pub streamelements: Option<ConsumerStreamElementsAPI>,
    pub youtube_playlist: Option<ConsumerYouTubePlaylistAPI>,
    control: Control, // exposed through Bot::stop
    config: config::BotConfig,
    pub start: chrono::DateTime<chrono::Utc>,
    pub commands: HashMap<String, Command>,
}

impl Bot {
    pub fn builder(control: Control) -> BotBuilder {
        BotBuilder {
            streamelements_api: None,
            youtube_api: None,
            control,
        }
    }

    #[inline]
    pub fn is_boss(&self, name: &str) -> bool {
        self.config.gym_staff.contains(name)
    }

    pub async fn run(mut self, dispatcher: Dispatcher) {
        
        let mut events = dispatcher.subscribe::<events::All>();

        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        // I give up, how do you do this without a clone? LULW
        self.join_configured_channels(&ready.nickname).await;

        self.send(
            &"moscowwbish".into_channel().unwrap(),
            "gachiHYPER I'M READY",
        )
        .await;

        while let Some(event) = events.next().await {
            match &*event {
                messages::AllCommands::Privmsg(msg) => {
                    self.handle_msg(msg).await;
                }
                _ => {}
            }
        }
    }

    pub fn stop(&mut self) {
        self.control.stop();
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>) {
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
            // reload the command
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
            self.send(&evt.channel, format!("FeelsDankMan 👉 {}", data.usage)).await;
            return;
        }

        let message = util::strip_prefix(&evt.data, "xD ");
        if let Some((command, args)) = util::find_command(&self.commands, message) {
            let args: mlua::Variadic<String> = if let Some(args) = args {
                mlua::Variadic::from_iter(args.into_iter().map(|it| it.to_owned()))
            } else {
                mlua::Variadic::new()
            };
            let response = (command.script).call_async::<mlua::Variadic<String>,String>(args).await;
            let response = match response {
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
                    channel, e
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
                        channel, e
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
                    channel, e
                );
            })
    }
}
