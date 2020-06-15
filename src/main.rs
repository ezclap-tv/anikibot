extern crate config;
extern crate log;
extern crate pretty_env_logger;
extern crate tokio;
extern crate twitchchat;

extern crate backend;

use std::collections::HashMap;
use std::convert::Into;
use std::time;

use log::{error, info};
use tokio::stream::StreamExt as _;
use twitchchat::{
    events, messages, Channel, Control, Dispatcher, IntoChannel, RateLimit, Runner, Status,
    UserConfig, Writer,
};

use backend::{StreamElementsAPI, StreamElementsConfig};

struct Secrets {
    name: String,
    oauth_token: String,
    stream_elements_jwt_token: String,
}

impl Secrets {
    fn get() -> Secrets {
        let mut secrets = config::Config::default();
        secrets.merge(config::File::with_name("secrets")).unwrap();
        let secrets = secrets.try_into::<HashMap<String, String>>().unwrap();

        let name = secrets.get("name").cloned().expect("Missing the bot name");
        let oauth_token = secrets
            .get("oauth_token")
            .cloned()
            .expect("Missing the oauth token");
        let stream_elements_jwt_token = secrets
            .get("stream_elements_jwt_token")
            .cloned()
            .expect("Missing the StreamElements JWT token");

        Secrets {
            name,
            oauth_token,
            stream_elements_jwt_token,
        }
    }
}

impl Into<UserConfig> for Secrets {
    fn into(self) -> UserConfig {
        twitchchat::UserConfig::builder()
            .name(&self.name)
            .token(&self.oauth_token)
            .build()
            .unwrap()
    }
}

struct BotConfig {
    channel: String,
}

impl BotConfig {
    fn get() -> BotConfig {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("bot")).unwrap();
        let config = config.try_into::<HashMap<String, String>>().unwrap();

        let channel = config.get("channel").cloned().unwrap();

        BotConfig { channel }
    }
}

struct Bot {
    api: StreamElementsAPI,
    writer: Writer,
    control: Control,
    config: BotConfig,
    start: time::Instant,
}

impl Bot {
    fn new(api: StreamElementsAPI, writer: Writer, control: Control) -> Bot {
        Bot {
            api,
            writer,
            control,
            config: BotConfig::get(),
            start: time::Instant::now(),
        }
    }

    async fn run(mut self, dispatcher: Dispatcher) {
        let channel = self.config.channel.clone().into_channel().unwrap();

        let mut events = dispatcher.subscribe::<events::All>();

        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        println!("Connected to {} as {}", &channel, &ready.nickname);
        self.writer.privmsg(&channel, "gachiHYPER").await.unwrap();
        self.writer.join(&channel).await.unwrap();

        while let Some(event) = events.next().await {
            match &*event {
                messages::AllCommands::Join(join) => {
                    if !self.handle_join(join).await {
                        return;
                    }
                }
                messages::AllCommands::Privmsg(msg) => {
                    if !self.handle_msg(msg).await {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    async fn handle_join(&mut self, evt: &messages::Join<'_>) -> bool {
        match &*evt.name.to_lowercase().trim() {
            "moscowwbish" => {
                let resp = format!("gachiHYPER @moscowwbish");
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            _ => {}
        };

        true
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>) -> bool {
        info!("received a new event: {:?}", &*evt.data);
        match &*evt.data {
            "xD" => {
                info!("command \"xD\" in channel {}", &evt.channel);
                let resp = format!("xD");
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            "xD test" => {
                info!("command \"xD test\" in channel {}", &evt.channel);
                let resp = format!(
                    "FeelsDankMan uptime {:.2?}",
                    time::Instant::now() - self.start
                );
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            cmd @ "xD whoami" => {
                info!("command {:?} in channel {}", cmd, &evt.channel);
                let resp = match self.api.channels().channel_id(&*evt.name).await {
                    Ok(id) => format!("monkaHmm your id is {}", id),
                    Err(e) => {
                        error!(
                            "Failed to fetch the channel id for the username {:?}: {}",
                            &evt.name, e
                        );
                        format!("sorry Pepega devs broke something")
                    }
                };
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            "xD stop" => {
                if &*evt.name == "moscowwbish" {
                    info!("command \"xD stop\" in channel {}", &evt.channel);
                    self.control.stop();
                    return false;
                }
            }
            rest => {
                error!("unknown command: {:?}", rest);
            }
        };

        true
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());

    let secrets = Secrets::get();

    info!("Initializing the stream_elements API.");
    let api = StreamElementsAPI::new(
        StreamElementsConfig::with_token(secrets.stream_elements_jwt_token.clone()).unwrap(),
    )
    .finalize()
    .await
    .unwrap();

    let bot = Bot::new(api, control.writer().clone(), control.clone()).run(dispatcher);

    info!("Connecting to twitch...");
    let conn = twitchchat::connect_tls(&secrets.into()).await.unwrap();

    let done = runner.run(conn);

    tokio::select! {
        _ = bot => { println!("bot stopped") },
        status = done => {
            match status {
                Ok(Status::Canceled) => {
                    println!("runner cancelled");
                },
                Ok(Status::Eof) => {
                    println!("got eof");
                },
                Ok(Status::Timeout) => {
                    println!("timed out");
                },
                Err(err) => {
                    panic!(format!("{}", err));
                }
            }
        }
    }
}
