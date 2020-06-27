use reqwest::Response;
use serde_json::{json, Value};

use super::api::{APIResult, StreamElementsAPI};

pub struct SongRequests<'a> {
    api: &'a StreamElementsAPI,
}

impl<'a> SongRequests<'a> {
    /// Creates a new `SongRequests` object.
    pub fn new(api: &'a StreamElementsAPI) -> Self {
        Self { api }
    }

    /// Retrieves the song request settings of the API user.
    #[inline(always)]
    pub async fn get_settings(&self) -> APIResult<Response> {
        self.api.get_method("songrequest", "settings").send().await
    }

    /// Retrieves the song request settings for the given `channel_id`.
    #[inline(always)]
    pub async fn get_public_settings(&self, channel_id: &str) -> APIResult<Response> {
        self.api
            .get_method_for_channel_id(channel_id, "songrequest", "settings/public")
            .send()
            .await
    }

    /// Retrieves the currently playing song.
    #[inline(always)]
    pub async fn current_song(&self) -> APIResult<Response> {
        self.api.get_method("songrequest", "playing").send().await
    }

    /// Returns the title of the currently playing song.
    pub async fn current_song_title(&self) -> APIResult<String> {
        Ok(self
            .current_song()
            .await?
            .json::<Value>()
            .await
            .map(|v| v["title"].as_str().unwrap().to_owned())
            .unwrap_or_else(|e| {
                log::error!("Couldn't decode the current song info: {}", e);
                String::from("Nothing is playing at the moment.")
            }))
    }

    /// Queues the given song in the given channel.
    pub async fn queue_song_in_channel(
        &self,
        channel_id: &str,
        song_url: &str,
    ) -> APIResult<Response> {
        self.api
            .post_method_for_channel_id(channel_id, "songrequest", "queue")
            .json(&json!({ "video": song_url }))
            .send()
            .await
    }

    /// Queues the given song in the API user's channel.
    #[inline(always)]
    pub async fn queue_song(&self, song_url: &str) -> APIResult<Response> {
        self.queue_song_in_channel(&self.api.channel_id(), song_url)
            .await
    }
}
