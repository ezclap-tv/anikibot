extern crate config;
extern crate tokio;
extern crate twitchchat;
// extern crate reqwest; // see Cargo.toml notes comment

use std::collections::HashMap;
use std::convert::Into;
use std::time;
use tokio::stream::StreamExt as _;
use twitchchat::{
    events, messages, Control, Dispatcher, Channel, IntoChannel, RateLimit, Runner, Status, Writer, UserConfig
};

struct Secrets {
    name: String,
    oauth_token: String,
}

impl Secrets {
    fn get() -> Secrets {
        let mut secrets = config::Config::default();
        secrets.merge(config::File::with_name("secrets")).unwrap();
        let secrets = secrets.try_into::<HashMap<String, String>>().unwrap();
    
        let name = secrets.get("name").cloned().unwrap();
        let oauth_token = secrets.get("oauth_token").cloned().unwrap();
    
        Secrets { name, oauth_token }
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
    channel: String
}

impl BotConfig {
    fn get() -> BotConfig {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("bot")).unwrap();
        let config = config.try_into::<HashMap<String, String>>().unwrap();

        let channel = config.get("channel").cloned().unwrap();

        BotConfig {
            channel
        }
    }
}

struct Bot {
    writer: Writer,
    control: Control,
    config: BotConfig,
    start: time::Instant
}

impl Bot {
    fn new(writer: Writer, control: Control) -> Bot {
        Bot {
            writer, control,
            config: BotConfig::get(),
            start: time::Instant::now()
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
                },
                messages::AllCommands::Privmsg(msg) => {
                    if !self.handle_msg(msg).await {
                        return;
                    }
                },
                _ => {}
            }
        }
    }

    async fn handle_join(&mut self, evt: &messages::Join<'_>) -> bool {
        match &*evt.name.to_lowercase().trim() {
            "moscowwbish" => {
                let resp = format!("gachiHYPER @moscowwbish");
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            },
            _ => {}
        };

        true
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>) -> bool {
        println!("{:?}", &*evt.data);
        match &*evt.data {
            "xD" => {
                println!("command \"xD\" in channel {}", &evt.channel);
                let resp = format!("xD");
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            },
            "xD test" => {
                println!("command \"xD test\" in channel {}", &evt.channel);
                let resp = format!("FeelsDankMan uptime {:.2?}", time::Instant::now() - self.start);
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            },
            "xD stop" => {
                if &*evt.name == "moscowwbish" {
                    println!("command \"xD stop\" in channel {}", &evt.channel);
                    self.control.stop();
                    return false;
                }
            }
            _ => {}
        };

        true
    }
}

#[tokio::main]
async fn main() {
    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());
    
    let bot = Bot::new(control.writer().clone(), control.clone())
        .run(dispatcher);

    let conn = twitchchat::connect_tls(&Secrets::get().into()).await.unwrap();
    
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
