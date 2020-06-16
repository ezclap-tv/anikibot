mod config;
use self::config::BotConfig;

use log::{error, info};
use reqwest::Error as ReqwestError;
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Writer};

use crate::{
    stream_elements::api::StreamElementsAPI,
    youtube::{YouTubePlaylistAPI, YouTubeVideo},
};

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
    &str[prefix.len()..str.len()]
}

pub struct Bot {
    api: StreamElementsAPI,
    yt_api: Option<YouTubePlaylistAPI>,
    writer: Writer,
    control: Control,
    config: config::BotConfig,
    start: chrono::DateTime<chrono::Utc>,
}

impl Bot {
    pub fn new(api: StreamElementsAPI, writer: Writer, control: Control) -> Bot {
        Bot {
            api,
            yt_api: None,
            writer,
            control,
            config: BotConfig::get(),
            start: chrono::Utc::now(),
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
                self.send(evt, "xD").await;
            }
            cmd @ "xD ping" => {
                info!("command {:?} in channel {}", cmd, &evt.channel);

                let uptime: chrono::Duration = chrono::Utc::now() - self.start;
                let resp = format!("FeelsDankMan uptime {}", duration_format(uptime));
                self.send(evt, resp).await;
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
                        "WAYTOODANK devs broke something".to_owned()
                    }
                };
                self.send(evt, resp).await;
            }
            cmd @ "xD song" => {
                info!("command {:?} in channel {}", cmd, &evt.channel);
                let resp = match self.api.song_requests().current_song_title().await {
                    Ok(song) => format!("CheemJam currently playing song is {}", song),
                    Err(e) => {
                        error!("Failed to fetch the current song title {}", e);
                        format!("WAYTOODANK devs broke something")
                    }
                };
                self.send(evt, resp).await;
            }
            cmd @ "xD stop" => {
                if &*evt.name == "moscowwbish" {
                    info!("command {:?} in channel {}", cmd, &evt.channel);
                    self.control.stop();
                    return false;
                }
            }
            cmd if cmd.starts_with("xD new playlist") => {
                info!("command {:?} in channel {}", cmd, &evt.channel);
                if !self.is_boss(&*evt.name) {
                    self.send(
                        evt,
                        "FeelsDnakMan Sorry, you don't have the permission to change playlists",
                    )
                    .await;
                    return true;
                }
                if self.yt_api.is_none() {
                    self.send(evt, "FeelsDnakMan Youtube API is not is not available")
                        .await;
                    return true;
                }
                let args: Vec<&str> = cmd.split_whitespace().collect();
                if args.len() < 4 {
                    self.send(&evt, "THATSREALLYTOODANK Missing the playlist URL")
                        .await;
                    return true;
                }

                match extract_playlist_id(&args[3]) {
                    Some(playlist_id) => self
                        .yt_api
                        .as_mut()
                        .map(|api| api.set_playlist(playlist_id)),
                    None => {
                        error!("Invalid playlist url: {}", args[3]);
                        self.send(
                            evt,
                            "cheemSad Couldn't parse the playlist URL from your input",
                        )
                        .await;
                        return true;
                    }
                };

                match args.get(4) {
                    Some(n) => match n.parse::<usize>() {
                        Ok(n) => self.yt_api.as_mut().map(|api| api.page_size(n)),
                        Err(e) => {
                            error!("Invalid number of videos to queue: {}", e);
                            self.send(evt, "cheemSad couldn't parse the number of videos to queue")
                                .await;
                            return true;
                        }
                    },
                    None => None,
                };

                match self.yt_api.as_mut().unwrap().get_playlist_videos().await {
                    Ok(videos) => match self.queue_videos(videos).await {
                        Ok(n) => {
                            self.send(evt, format!("Successfully queued {} song(s)", n))
                                .await
                        }
                        Err(errors) => {
                            error!("Failed to queue n videos: {}", errors.len());
                            for e in errors {
                                error!("=> Error: {}", e);
                            }
                            self.send(evt, "THATSREALLYTOODANK failed to queue the playlist")
                                .await;
                        }
                    },
                    Err(e) => {
                        error!("Failed to retrieve the videos in the playlist: {}", e);
                        self.send(evt, "WAYTOODANK devs broke something").await;
                    }
                }
            }
            rest if rest.starts_with("xD") => {
                info!("unknown command {:?} in channel {}", rest, &evt.channel);
                let rest = strip_prefix(rest, "xD ");
                self.send(evt, format!("WAYTOODANK 👉 UNKNOWN COMMAND \"{}\"", rest))
                    .await;
            }
            _ => { /* not a command, just ignore 4Head */ }
        };

        true
    }

    async fn send<S: Into<String>>(&mut self, evt: &messages::Privmsg<'_>, message: S) {
        self.writer
            .privmsg(&evt.channel, message.into())
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Caught a critical error while sending a response to the channel {}: {:?}",
                    &evt.channel, e
                );
            })
    }

    async fn queue_videos(&self, videos: Vec<YouTubeVideo>) -> Result<usize, Vec<ReqwestError>> {
        let mut queued = 0;
        let mut errors = vec![];
        for (i, v) in videos.into_iter().enumerate() {
            let url = v.into_url();
            info!("Attempting to queue song #{}: {}", i, url);
            match self.api.song_requests().queue_song(&url).await {
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
