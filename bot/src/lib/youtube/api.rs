use super::{
    communication::spawn_api_thread,
    consumer::ConsumerYouTubePlaylistAPI,
    data::{PlaylistPage, Videos, YouTubeVideo},
};
use crate::{BackendError, YouTubeAPIConfig};
use reqwest::Client;
use serde_json::Value;
use tokio::runtime;

/// The base part of the YouTube playlist API.
pub const YOUTUBE_API_URL: &str =
    "https://www.googleapis.com/youtube/v3/playlistItems?part=contentDetails";

/// Provides a Rust interface to the YouTube Playlist API.
pub struct YouTubePlaylistAPI {
    client: Client,
    api_key: String,
    pub(crate) items_per_page: usize,
    pub(crate) number_of_videos: Option<usize>,
    // XXX: Consider using an Arc if this gets used a lot.
    playlist_id: Option<String>,
    next_page: String,
}

/// Ensures that the API is properly configured.
pub struct YouTubePlaylistAPIGuard {
    api: YouTubePlaylistAPI,
}

impl YouTubePlaylistAPIGuard {
    /// Stars the API thread and returns its sender and thread handle.
    pub fn start(
        self,
        runtime: runtime::Handle,
    ) -> (ConsumerYouTubePlaylistAPI, std::thread::JoinHandle<()>) {
        let (tx, handle) = spawn_api_thread(self.api, runtime);
        (ConsumerYouTubePlaylistAPI::new(tx), handle)
    }
}

impl YouTubePlaylistAPI {
    /// Creates a new `YouTubePlaylistAPI` instance wrapped in the guard type.
    /// To obtain a usable API object, the user must call [`start()`].
    ///
    /// [`start()`]: YouTubePlaylistAPIGuard::start
    pub fn with_api_key(api_key: String) -> YouTubePlaylistAPIGuard {
        YouTubePlaylistAPIGuard {
            api: Self {
                api_key,
                playlist_id: None,
                number_of_videos: None,
                client: Client::new(),
                items_per_page: 50,
                next_page: String::new(),
            },
        }
    }

    /// Returns the current configuration of the api object.
    pub fn get_config(&self) -> YouTubeAPIConfig {
        YouTubeAPIConfig {
            number_of_videos: self.number_of_videos,
            playlist_id: self.playlist_id.clone(),
            items_per_page: self.items_per_page,
            next_page: self.next_page.clone(),
        }
    }

    /// Sets the request page size to the given value.
    #[inline(always)]
    pub fn page_size(&mut self, items_per_page: usize) {
        self.items_per_page = items_per_page;
    }

    /// Changes the current playlist.
    #[inline]
    pub fn set_playlist(&mut self, playlist_id: String) {
        log::info!("Switched to a new playlist id: {}", playlist_id);
        self.playlist_id = Some(playlist_id);
        self.next_page = String::new();
        self.number_of_videos = None;
    }

    /// Returns the number of videos in the playlist.
    #[inline(always)]
    pub fn number_of_videos(&self) -> Option<usize> {
        self.number_of_videos
    }

    /// Returns the id of the current playlist.
    #[inline(always)]
    pub fn current_playlist(&self) -> Option<&str> {
        self.playlist_id.as_ref().map(|s| &s[..])
    }

    /// Returns the next batch of videos in the playlist.
    pub async fn get_playlist_videos(&mut self) -> Result<Videos, BackendError> {
        if self.playlist_id.is_some() {
            Ok(self.get_next_page().await?.videos)
        } else {
            Err(BackendError::from("Missing the playlist id.".to_owned()))
        }
    }

    async fn get_next_page(&mut self) -> Result<PlaylistPage, BackendError> {
        self.get_page(format!(
            "{}&playlistId={}&maxResults={}&key={}&pageToken={}",
            YOUTUBE_API_URL,
            self.playlist_id.as_ref().unwrap(),
            self.items_per_page,
            self.api_key,
            self.next_page,
        ))
        .await
        .map(|p| {
            self.next_page = p.next_page_token.clone().unwrap_or_else(String::new);
            p
        })
    }

    // TODO: make this panic-safe
    async fn get_page(&mut self, url: String) -> Result<PlaylistPage, BackendError> {
        let result = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(BackendError::from)?
            .json::<Value>()
            .await
            .map_err(BackendError::from)?;

        if let Some(error) = result.get("error") {
            log::error!("Failed to get the playlist: `{}`\n{:#?}", url, error);
            return Err(BackendError::from(format!(
                "YouTubePlaylistAPI Error: {}",
                error["message"]
            )));
        }

        self.number_of_videos = result["pageInfo"]["totalResults"]
            .as_u64()
            .map(|u| u as usize);

        Ok(PlaylistPage {
            kind: result["kind"].as_str().unwrap().to_owned(),
            next_page_token: result
                .get("nextPageToken")
                .map(|v| v.as_str().unwrap().to_owned()),
            videos: result["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| YouTubeVideo {
                    id: v["contentDetails"]["videoId"].as_str().unwrap().to_owned(),
                })
                .collect(),
        })
    }
}
