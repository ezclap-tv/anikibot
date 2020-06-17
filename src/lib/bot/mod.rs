
pub mod config;
pub mod util;
pub mod command;

use log::{error, info};
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel};

use crate::{
    stream_elements::consumer::ConsumerStreamElementsAPI,
    youtube::YouTubePlaylistAPI,
};
use command::Command;

use std::collections::HashMap;

/* Previously had commands: help, ping, ping uptime, whoami, stop, song, song queue*/

pub struct BotBuilder {
    streamelements_api: Option<ConsumerStreamElementsAPI>,
    youtube_api: Option<YouTubePlaylistAPI>,
    commands: Option<HashMap<String, Command>>,
    control: Control,
}

impl BotBuilder {
    pub fn add_streamelements_api(self, streamelements_api: ConsumerStreamElementsAPI) -> Self {
        BotBuilder {
            streamelements_api: Some(streamelements_api),
            ..self
        }
    }

    pub fn add_youtube_api(self, youtube_api: YouTubePlaylistAPI) -> Self {
        BotBuilder {
            youtube_api: Some(youtube_api),
            ..self
        }
    }

    pub fn add_commands(self, commands: HashMap<String, Command>) -> Self {
        BotBuilder {
            commands: Some(commands),
            ..self
        }
    }

    pub fn build(self) -> Bot {
        Bot {
            streamelements: self.streamelements_api,
            youtube_playlist: self.youtube_api,
            control: self.control,
            config: config::BotConfig::get(),
            start: chrono::Utc::now(),
            commands: self.commands.unwrap_or_else(|| HashMap::new()),
        }
    }
}

pub struct Bot {
    pub streamelements: Option<ConsumerStreamElementsAPI>,
    pub youtube_playlist: Option<YouTubePlaylistAPI>,
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
            commands: None,
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
        for channel in &self.config.channels.clone() {
            info!("Connected to {} as {}", &channel, &ready.nickname);
            self.join(&channel).await;
        }
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

        let message = util::strip_prefix(&evt.data, "xD ");
        if let Some((command, args)) = util::find_command(&self.commands, message) {
            let response = (command.factory)(self, evt, args).await;
            self.send(&evt.channel, &response).await;
        } else {
            self.send(&evt.channel, "WAYTOODANK 👉 Unknown command!")
                .await;
        }
    }

    async fn join(&mut self, channel: &str) {
        self.control.writer()
            .join(channel)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Caught a critical error while joining a channel {}: {:?}",
                    channel, e
                );
            })
    }

    async fn send<S: Into<String>>(&mut self, channel: &str, message: S) {
        self.control.writer()
            .privmsg(channel, message.into())
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Caught a critical error while sending a response to the channel {}: {:?}",
                    channel, e
                );
            })
    }
}
