mod config;
use self::config::BotConfig;

use log::{error, info};
use reqwest::Error as ReqwestError;
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Writer};
use serde_json::{json, Value};

use crate::{
    stream_elements::api::StreamElementsAPI,
    youtube::{YouTubePlaylistAPI, YouTubeVideo},
};

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

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
    if !str.starts_with(prefix) {
        &str[..]
    } else {
        &str[prefix.len()..str.len()]
    }
}

fn find_command<'a>(
    commands: &HashMap<String, Command>,
    message: &'a str,
) -> Option<(CommandData, Option<Vec<&'a str>>)> {
    // split the message by whitespace, collect into a vector
    let tokens = message.split_whitespace().collect::<Vec<&str>>();
    // next_commands holds the subcommands of the node we're looking at
    let mut next_commands = commands;
    for i in 0..tokens.len() {
        if let Some(command) = next_commands.get(tokens[i]) {
            // in this case, we may have gotten a command, or a subcommand
            // first we grab the next token if we can (not out of bounds for the vector)
            let commands = command.commands.as_ref();
            let data = command.data.as_ref();

            let next = if i + 1 < tokens.len() {
                Some(tokens[i + 1])
            } else {
                None
            };

            // if there is another token AND this command has subcommands AND the token is in the list of subcommands
            if next.is_some() && commands.is_some() && commands.unwrap().contains_key(next.unwrap())
            {
                // then we set the next_commands to commands
                next_commands = commands.unwrap();
                // and continue iterating
                continue;
            }

            // otherwise, we check if we got any command data
            if data.is_some() {
                // if so, this is a command

                // we're using the extra tokens as args:
                // 1. split the tokens at the next index
                let mut args: Option<Vec<&str>> = None;
                if tokens.len() - i > 0 {
                    let (_, right) = tokens.split_at(i + 1);
                    // if there are any tokens on the right side
                    if right.len() > 0 {
                        // slap those suckers into a vec
                        args = Some(right.to_vec())
                    }
                }
                // then return the command data and arguments
                return Some((data.cloned().unwrap(), args));
            } else {
                // otherwise, this is an unknown command
                return None;
            }
        }
    }

    None
}

async fn help(bot: &mut Bot, _: &messages::Privmsg<'_>, args: Option<Vec<&str>>) -> String {
    let commands = &bot.commands;
    if args.is_some() {
        if let Some((command, _)) = find_command(commands, &args.unwrap().join(" ")) {
            return format!("{}", command.help);
        }
    }

    let mut resp = format!("FeelsDankMan 👉 try other commands: ");
    let keys = commands.keys().into_iter().collect::<Vec<&String>>();
    for i in 0..keys.len() {
        // temporarily don't display "sensitive" commands
        if keys[i] == "stop" {
            continue;
        }
        resp += keys[i];
        if i + 1 < keys.len() {
            resp += ", "
        }
    }
    return resp;
}

async fn ping_uptime(bot: &mut Bot, _: &messages::Privmsg<'_>, _: Option<Vec<&str>>) -> String {
    let uptime: chrono::Duration = chrono::Utc::now() - bot.start;
    format!("FeelsDankMan uptime {}", duration_format(uptime))
}

async fn ping(_: &mut Bot, _: &messages::Privmsg<'_>, _: Option<Vec<&str>>) -> String {
    format!("FeelsDankMan 👍 Pong!")
}

async fn whoami(bot: &mut Bot, evt: &messages::Privmsg<'_>, _: Option<Vec<&str>>) -> String {
    match bot.se_api.channels().channel_id(&*evt.name).await {
        Ok(id) => format!("monkaHmm your id is {}", id),
        Err(e) => {
            error!(
                "Failed to fetch the channel id for the username {:?}: {}",
                &evt.name, e
            );
            format!("WAYTOODANK devs broke something")
        }
    }
}

async fn stop(bot: &mut Bot, evt: &messages::Privmsg<'_>, _: Option<Vec<&str>>) -> String {
    if bot.is_boss(&evt.name) {
        bot.stop();
        return String::new();
    }

    return String::new();
}

async fn song(bot: &mut Bot, _: &messages::Privmsg<'_>, _: Option<Vec<&str>>) -> String {
    let song = bot.se_api.song_requests().current_song().await;
    let song = if let Ok(song) = song { song } else {
        return format!("cheemSad Unable to retrieve currently playing song!");
    };
    let song = song.json::<Value>().await;
    let song = match song { 
        Ok(song) => song,
        Err(e) => {
            error!("{}", e);
            return format!("WAYTOODANK devs broke something")
        }
    };

    let title = song["title"].as_str().unwrap();
    let video_id = song["videoId"].as_str().unwrap();
    format!("CheemJam Currently playing song: {} CheemJam https://youtu.be/{}", title, video_id)
}

async fn song_queue(bot: &mut Bot, evt: &messages::Privmsg<'_>, args: Option<Vec<&str>>) -> String {
    if !bot.is_boss(&evt.name) {
        return format!("FeelsDnakMan Sorry, you don't have the permission to queue songs");
    }
    let yt_api = if let Some(api) = &mut bot.yt_api {
        api
    } else {
        return format!("FeelsDnakMan Youtube API is not available");
    };
    // the extract_playlist_id function searches for the substring "list="
    // so we can do the same here
    let args = if let Some(args) = args {
        args
    } else {
        return format!("THATSREALLYTOODANK No arguments provided!");
    };

    match args.get(0) {
        Some(arg) => {
            match extract_playlist_id(arg) {
                Some(playlist_id) => yt_api.set_playlist(playlist_id),
                None => {
                    error!("Invalid playlist url: {}", arg);
                    return format!("cheemSad Couldn't parse the playlist URL from your input");
                }
            };
        }
        None => {
            return format!("THATSREALLYTOODANK No youtube playlist URL");
        }
    }

    match args.get(1) {
        Some(n) => match n.parse::<usize>() {
            Ok(n) => {
                yt_api.page_size(n);
            }
            Err(e) => {
                error!("Invalid number of videos to queue: {}", e);
                return format!("cheemSad Failed to queue the playlist");
            }
        },
        None => (),
    };

    match yt_api.get_playlist_videos().await {
        Ok(videos) => match bot.queue_videos(videos).await {
            Ok(n) => {
                return format!("Successfully queued {} song(s)", n);
            }
            Err(errors) => {
                error!("Failed to queue n videos: {}", errors.len());
                for e in errors {
                    error!("=> Error: {}", e);
                }
                return format!("cheemSad Failed to queue the playlist");
            }
        },
        Err(e) => {
            error!("Failed to retrieve the videos in the playlist: {}", e);
            return format!("cheemSad Failed to queue the playlist");
        }
    }
}

type ResponseFactory = for<'a> fn(
    &'a mut Bot,
    evt: &'a messages::Privmsg<'_>,
    Option<Vec<&'a str>>,
) -> Pin<Box<dyn Future<Output = String> + 'a>>;

#[derive(Clone)]
pub struct CommandData {
    /// Contains info about command usage
    help: String,
    /// Pointer to function with command logic
    /// This should eventually be replaced by a script
    factory: ResponseFactory,
}

pub struct Command {
    commands: Option<HashMap<String, Command>>,
    data: Option<CommandData>,
}

pub struct Bot {
    pub se_api: StreamElementsAPI,
    pub yt_api: Option<YouTubePlaylistAPI>,
    writer: Writer,   // exposed through Bot::send
    control: Control, // exposed through Bot::stop
    config: config::BotConfig,
    pub start: chrono::DateTime<chrono::Utc>,
    pub commands: HashMap<String, Command>,
    stopped: bool,
}

impl Bot {
    pub fn new(api: StreamElementsAPI, writer: Writer, control: Control) -> Bot {
        let commands: HashMap<String, Command> = vec![
            ("help".into(), Command {
                commands: None,
                data: Some(CommandData {
                    help: "good one 4Head".into(),
                    factory: |b,m,a| { Box::pin(help(b,m,a)) },
                }),
            }),
            ("ping".into(), Command {
                commands: Some(vec![
                    ("uptime".into(),
                    Command {
                        commands: None,
                        data: Some(CommandData {
                            help: "Outputs the bot uptime".into(),
                            factory: |b,m,a| { Box::pin(ping_uptime(b,m,a)) },
                        }),
                    })
                ].into_iter().collect()),
                data: Some(CommandData {
                    help: "Pong!".into(),
                    factory: |b,m,a| { Box::pin(ping(b,m,a)) },
                }),
            }),
            ("whoami".into(), Command {
                commands: None,
                data: Some(CommandData {
                    help: "monkaS Returns your StreamElements account id".into(),
                    factory: |b,m,a| { Box::pin(whoami(b,m,a)) }
                })
            }),
            ("stop".into(), Command {
                commands: None,
                data: Some(CommandData {
                    help: "Stops the bot".into(),
                    factory: |b,m,a| { Box::pin(stop(b,m,a)) }
                })
            }),
            ("song".into(), Command {
                commands: Some(vec![
                    ("queue".into(), Command {
                        commands: None,
                        data: Some(CommandData {
                            help: "FeelsDankMan Adds ~50 videos from a YouTube playlist to the StreamElements song queue. Usage: \"playlist queue <youtube playlist link>\"".into(),
                            factory: |b,m,a| { Box::pin(song_queue(b,m,a))}
                        })
                    })
                ].into_iter().collect()),
                data: Some(CommandData {
                    help: "Shows the currently playing song".into(),
                    factory: |b,m,a| { Box::pin(song(b,m,a)) }
                })
            })
        ].into_iter().collect();

        Bot {
            se_api: api,
            yt_api: None,
            writer,
            control,
            config: BotConfig::get(),
            start: chrono::Utc::now(),
            commands,
            stopped: false,
        }
    }

    pub fn with_youtube_api(
        api: StreamElementsAPI,
        yt_api: YouTubePlaylistAPI,
        writer: Writer,
        control: Control,
    ) -> Bot {
        Self {
            yt_api: Some(yt_api),
            ..Bot::new(api, writer, control)
        }
    }

    #[inline]
    pub fn is_boss(&self, name: &str) -> bool {
        self.config.gym_staff.contains(name)
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
                    if !self.stopped {
                        self.handle_msg(msg).await;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn stop(&mut self) {
        self.control.stop();
        self.stopped = true;
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

        let message = strip_prefix(&evt.data, "xD ");
        if let Some((command, args)) = find_command(&self.commands, message) {
            let response = (command.factory)(self, evt, args).await;
            self.send(&evt.channel, &response).await;
        } else {
            self.send(&evt.channel, "WAYTOODANK 👉 Unknown command!")
                .await;
        }
    }

    async fn send<S: Into<String>>(&mut self, channel: &str, message: S) {
        self.writer
            .privmsg(channel, message.into())
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Caught a critical error while sending a response to the channel {}: {:?}",
                    channel, e
                );
            })
    }

    async fn queue_videos(&self, videos: Vec<YouTubeVideo>) -> Result<usize, Vec<ReqwestError>> {
        let mut queued = 0;
        let mut errors = vec![];
        for (i, v) in videos.into_iter().enumerate() {
            let url = v.into_url();
            info!("Attempting to queue song #{}: {}", i, url);
            match self.se_api.song_requests().queue_song(&url).await {
                Ok(r) => {
                    queued += 1;
                    info!(
                        "Successfully queued `{}`",
                        r.json::<serde_json::Value>()
                            .await
                            .unwrap()
                            .get("title")
                            .unwrap()
                            .as_str()
                            .unwrap()
                    )
                }
                Err(e) => {
                    error!(
                        "Failed to queue the song with url={}, \nError was: {}",
                        url, e
                    );
                    errors.push(e);
                }
            }
        }

        info!("Successfully queued {} song(s)", queued);
        if errors.is_empty() {
            Ok(queued)
        } else {
            Err(errors)
        }
    }
}

fn extract_playlist_id(url: &str) -> Option<String> {
    info!("{}", url);
    if let Some(start) = url.find("list=").map(|idx| idx + 5) {
        let mut end = url.len();
        for (i, ch) in url.chars().enumerate().skip(start + 1) {
            if ch == '&' {
                end = i;
                break;
            }
        }
        if start < end {
            return Some(url[start..end].to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_extractor() {
        let playlists = vec![
            "",
            "OMEGALUL",
            "list=",
            "=list",
            "list=PL96Hybk1gPsgwPnEQ9fj1yNBtUdLnLloB",
            "https://www.youtube.com/watch?v=gA3nKW0JsM8&list=PL96Hybk1gPsgwPnEQ9fj1yNBtUdLnLloB&index=31",
        ];
        let expected = vec![
            None,
            None,
            None,
            None,
            Some("PL96Hybk1gPsgwPnEQ9fj1yNBtUdLnLloB".to_owned()),
            Some("PL96Hybk1gPsgwPnEQ9fj1yNBtUdLnLloB".to_owned()),
        ];

        for (i, (p, e)) in playlists.iter().zip(expected.into_iter()).enumerate() {
            let result = extract_playlist_id(p);
            assert_eq!(result, e, "[TEST #{}] Failed to extract the playlist", i);
        }
    }
}
