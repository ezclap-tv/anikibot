mod config;
use self::config::BotConfig;

use log::{error, info};
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Writer};

use crate::stream_elements::api::StreamElementsAPI;

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

// TODO move this elsewhere
fn duration_format(duration: chrono::Duration) -> String {
    let mut output = String::from("");

    let days = duration.num_days();
    if days > 0 {
        output += &format!("{} days ", days);
    }
    let hours = duration.num_hours();
    if hours > 0 {
        output += &format!("{} hours ", hours - days * 24);
    }
    let minutes = duration.num_minutes();
    if minutes > 0 && days <= 0 {
        output += &format!("{} minutes ", minutes - hours * 60);
    }
    let seconds = duration.num_seconds();
    if seconds > 0 && hours <= 0 {
        output += &format!("{} seconds", seconds - minutes * 60);
    }

    output
}

fn strip_prefix<'a>(str: &'a str, prefix: &str) -> &'a str {
    if !str.starts_with(prefix) { &str[..] }
    else { &str[prefix.len()..str.len()] }
}

fn find_command(commands: &HashMap<String, Command>, message: &str) -> Option<CommandData> {
    // 1. split the message by whitespace, collect into a vector
    let tokens = message.split_whitespace().collect::<Vec<&str>>();
    // next_commands holds the subcommands of the node we're looking at
    let mut next_commands = commands;
    for i in 0..tokens.len() {
        // if we can't find the token, exit
        if !next_commands.contains_key(tokens[i]) {
            return None;
        }
        // otherwise, match what we got
        match next_commands.get(tokens[i]).unwrap() {
            Command::Leaf { data } => {
                // in this case, we got a command with no subcommands,
                // just return it 4Head
                return Some(data.clone());
            }
            Command::Branch { commands, data } => {
                // in this case, we may have gotten a command, or a subcommand
                // first we grab the next token if we can (not out of bounds for the vector)
                let next = if i + 1 < tokens.len() {
                    Some(tokens[i + 1])
                } else {
                    None
                };

                // if there is another token, and this command has a subcommand with that name
                if next.is_some() && commands.contains_key(next.unwrap()) {
                    // then we set the next_commands to commands
                    next_commands = commands;
                    // and continue iterating
                    continue;
                } else {
                    // otherwise, we check if we got any command data
                    if data.is_none() {
                        // if not, this is an unknown command
                        return None;
                    } else {
                        // if so, this is a command, and we can return its data
                        return Some(data.clone().unwrap());
                    }
                }
            }
        }
    }

    None
}

async fn help(bot: &mut Bot, msg: &messages::Privmsg<'_>) -> (String, bool){
    let commands = bot.get_commands();
    let msg = strip_prefix(&msg.data, "xD help ");
    if let Some(command) = find_command(commands, msg) {
        (format!("{}", command.help), true)
    } else {
        let mut resp = format!("FeelsDankMan 👉 try other commands: ");
        let keys = commands.keys().into_iter().collect::<Vec<&String>>();
        for i in 0..keys.len() {
            // temporarily don't display "sensitive" commands
            if keys[i] == "stop" { continue; }
            resp += keys[i];
            if i+1 < keys.len() { resp += ", " }
        }
        (resp, true)
    }
}

async fn ping_uptime(bot: &mut Bot, _: &messages::Privmsg<'_>) -> (String, bool) {
    let uptime: chrono::Duration = chrono::Utc::now() - *bot.get_start();
    (format!("FeelsDankMan uptime {}", duration_format(uptime)), true)
}

async fn ping(_: &mut Bot, _: &messages::Privmsg<'_>) -> (String, bool) {
    (format!("FeelsDankMan 👍 Pong!"), true)
}

async fn whoami(bot: &mut Bot, msg: &messages::Privmsg<'_>) -> (String, bool) {
    match bot.get_streamelements_api().channels().channel_id(&*msg.name).await {
        Ok(id) => (format!("monkaHmm your id is {}", id), true),
        Err(e) => {
            error!(
                "Failed to fetch the channel id for the username {:?}: {}",
                &msg.name, e
            );
            (format!("WAYTOODANK devs broke something"), true)
        }
    }
}

async fn stop(_: &mut Bot, msg: &messages::Privmsg<'_>) -> (String, bool) {
    if &*msg.name == "moscowwbish" {
        return (String::new(), false);
    }

    return (String::new(), true);
}

async fn song(bot: &mut Bot, _: &messages::Privmsg<'_>) -> (String, bool) {
    match bot.get_streamelements_api().song_requests().current_song_title().await {
        Ok(song) => (format!("CheemJam currently playing song is {}", song), true),
        Err(e) => {
            error!("Failed to fetch the current song title {}", e);
            (format!("WAYTOODANK devs broke something"), true)
        }
    }
}

type ResponseFactory = for<'a> fn(&'a mut Bot, &'a messages::Privmsg) -> Pin<Box<dyn Future<Output = (String, bool)> + 'a>>;

#[derive(Clone)]
pub struct CommandData {

    /// Contains info about command usage
    help: String,
    /// Pointer to function with command logic
    /// This should eventually be replaced by a script
    factory: ResponseFactory,
}

pub enum Command {
    Leaf {
        data: CommandData,
    },
    Branch {
        commands: HashMap<String, Command>,
        data: Option<CommandData>,
    },
}

pub struct Bot {
    api: StreamElementsAPI,
    writer: Writer,
    control: Control,
    config: config::BotConfig,
    start: chrono::DateTime<chrono::Utc>,

    commands: HashMap<String, Command>,
}

impl Bot {
    pub fn new(api: StreamElementsAPI, writer: Writer, control: Control) -> Bot {
        /* command tree:
            xD
            |__<empty>
            |__help
            |__ping
            |  |__uptime
            |__whoami
            |__stop
            |__song
        */

        let commands: HashMap<String, Command> = vec![
            ("help".into(), Command::Leaf {
                data: CommandData {
                    help: "good one 4Head".into(),
                    factory: |b,m| { Box::pin(help(b,m)) },
                },
            }),
            ("ping".into(), Command::Branch {
                commands: vec![
                    ("uptime".into(),
                    Command::Leaf {
                        data: CommandData {
                            help: "Outputs the bot uptime".into(),
                            factory: |b,m| { Box::pin(ping_uptime(b,m)) },
                        },
                    })
                ].into_iter().collect(),
                data: Some(CommandData {
                    help: "Pong!".into(),
                    factory: |b,m| { Box::pin(ping(b,m)) },
                }),
            }),
            ("whoami".into(), Command::Leaf {
                data: CommandData {
                    help: "monkaS Returns your StreamElements account id".into(),
                    factory: |b,m| { Box::pin(whoami(b,m)) }
                }
            }),
            ("stop".into(), Command::Leaf {
                data: CommandData {
                    help: "Stops the bot".into(),
                    factory: |b,m| { Box::pin(stop(b,m)) }
                }
            }),
            ("song".into(), Command::Leaf {
                data: CommandData {
                    help: "Shows the currently playing song".into(),
                    factory: |b,m| { Box::pin(song(b,m)) }
                }
            })
        ].into_iter().collect();

        Bot {
            api,
            writer,
            control,
            config: BotConfig::get(),
            start: chrono::Utc::now(),
            commands,
        }
    }

    pub async fn run(mut self, dispatcher: Dispatcher) {
        let channel = self.config.channel.clone().into_channel().unwrap();

        let mut events = dispatcher.subscribe::<events::All>();

        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();

        info!("Connected to {} as {}", &channel, &ready.nickname);
        self.writer
            .privmsg(&channel, "gachiHYPER I'M READY")
            .await
            .unwrap();
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

    pub fn get_streamelements_api(&mut self) -> &StreamElementsAPI {
        &self.api
    }
    pub fn get_writer(&mut self) -> &Writer {
        &self.writer
    }
    pub fn get_config(&mut self) -> &BotConfig {
        &self.config
    }
    pub fn get_start(&mut self) -> &chrono::DateTime<chrono::Utc> {
        &self.start
    }
    pub fn get_commands(&mut self) -> &HashMap<String, Command> {
        &self.commands
    }

    async fn handle_msg(&mut self, evt: &messages::Privmsg<'_>) -> bool {
        if !evt.data.starts_with("xD") {
            return true;
        }

        // hardcoded "xD" response because it needs to exist
        if evt.data.trim() == "xD" {
            self.writer.privmsg(&evt.channel, "xD").await.unwrap();
            return true;
        }

        let message = strip_prefix(&evt.data, "xD ");
        if let Some(command) = find_command(&self.commands, message) {
            let (response, continue_running) = (command.factory)(self, evt).await;
            if !continue_running {
                self.control.stop();
                return false;
            } else {
                self.writer.privmsg(&evt.channel, &response).await.unwrap();
                return true;
            }
        } else {
            self.writer.privmsg(&evt.channel, "WAYTOODANK 👉 Unknown command!").await.unwrap();
            return true;
        }
    }
}
