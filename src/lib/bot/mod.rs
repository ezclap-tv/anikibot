
mod config;
use self::config::BotConfig;

use tokio::stream::StreamExt as _;
use crate::stream_elements::api::StreamElementsAPI;
use twitchchat::{
    events, messages, Control, Dispatcher, IntoChannel, Writer,
};
use log::{info, error};

// TODO move this elsewhere
fn duration_format(duration: chrono::Duration) -> String {
    let mut output = String::from("");
    
    let days = duration.num_days();
    if days > 0 { output += &format!("{} days ", days); }
    let hours = duration.num_hours();
    if hours > 0 { output += &format!("{} hours ", hours - days*24); }
    let minutes = duration.num_minutes();
    if minutes > 0 && days <= 0 { output += &format!("{} minutes ", minutes - hours*60); }
    let seconds = duration.num_seconds();
    if seconds > 0 && hours <= 0 { output += &format!("{} seconds", seconds - minutes*60); }
    
    output
}

fn strip_prefix<'a>(str: &'a str, prefix: &str) -> &'a str {
    &str[prefix.len()..str.len()]
}

pub struct Bot {
    api: StreamElementsAPI,
    writer: Writer,
    control: Control,
    config: config::BotConfig,
    start: chrono::DateTime<chrono::Utc>,
}

impl Bot {
    pub fn new(api: StreamElementsAPI, writer: Writer, control: Control) -> Bot {
        
        Bot {
            api,
            writer,
            control,
            config: BotConfig::get(),
            start: chrono::Utc::now(),
        }
    }

    pub async fn run(mut self, dispatcher: Dispatcher) {
        let channel = self.config.channel.clone().into_channel().unwrap();

        let mut events = dispatcher.subscribe::<events::All>();

        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        info!("Connected to {} as {}", &channel, &ready.nickname);
        self.writer.privmsg(&channel, "gachiHYPER I'M READY").await.unwrap();
        self.writer.join(&channel).await.unwrap();

        while let Some(event) = events.next().await {
            match &*event {
                messages::AllCommands::Privmsg(msg) => {
                    if !self.handle_msg(msg).await {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>) -> bool {
        match &*evt.data {
            cmd @ "xD" => {
                info!("command {:?} in channel {}", cmd, &evt.channel);
                let resp = format!("xD");
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            cmd @ "xD ping" => {
                info!("command {:?} in channel {}", cmd, &evt.channel);

                let uptime: chrono::Duration = chrono::Utc::now() - self.start;
                let resp = format!("FeelsDankMan uptime {}", duration_format(uptime));
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
                        format!("WAYTOODANK devs broke something")
                    }
                };
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            cmd @ "xD stop" => {
                if &*evt.name == "moscowwbish" {
                    info!("command {:?} in channel {}", cmd, &evt.channel);
                    self.control.stop();
                    return false;
                }
            }
            rest if rest.starts_with("xD") => {
                info!("unknown command {:?} in channel {}", rest, &evt.channel);
                let rest = strip_prefix(rest, "xD ");
                let resp = format!("WAYTOODANK 👉 UNKNOWN COMMAND \"{}\"", rest);
                self.writer.privmsg(&evt.channel, &resp).await.unwrap();
            }
            _ => { /* not a command, just ignore 4Head */ }
        };

        true
    }
}