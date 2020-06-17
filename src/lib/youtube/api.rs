use super::{
    communication::spawn_api_thread,
    consumer::ConsumerYouTubePlaylistAPI,
    data::{PlaylistPage, Videos, YouTubeVideo},
};
use crate::{BackendError, BoxedError, YouTubeAPIConfig};
use log::info;
use reqwest::{Client, Error as ReqwestError};
use serde_json::Value;
use tokio::runtime;

pub const YOUTUBE_API_URL: &'static str =
    "https://www.googleapis.com/youtube/v3/playlistItems?part=contentDetails";

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
    pub fn new(api_key: String) -> YouTubePlaylistAPIGuard {
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

    pub fn get_config(&self) -> YouTubeAPIConfig {
        YouTubeAPIConfig {
            number_of_videos: self.number_of_videos,
            playlist_id: self.playlist_id.clone(),
            items_per_page: self.items_per_page,
            next_page: self.next_page.clone(),
        }
    }

    #[inline(always)]
    pub fn page_size(&mut self, items_per_page: usize) {
        self.items_per_page = items_per_page;
    }

    #[inline]
    pub fn set_playlist(&mut self, playlist_id: String) {
        info!("Switched to a new playlist id: {}", playlist_id);
        self.playlist_id = Some(playlist_id);
    }

    #[inline(always)]
    pub fn number_of_videos(&self) -> Option<usize> {
        self.number_of_videos
    }

    #[inline(always)]
    pub fn current_playlist(&self) -> Option<&str> {
        self.playlist_id.as_ref().map(|s| &s[..])
    }

    pub async fn get_playlist_videos(&mut self) -> Result<Videos, BackendError> {
        if self.playlist_id.is_some() {
            Ok({
                if self.next_page.is_empty() {
                    self.get_first_page()
                        .await
                        .map(|p| {
                            self.next_page =
                                p.next_page_token.clone().unwrap_or_else(|| String::new());
                            p
                        })
                        .map_err(|e| BackendError::from(Box::new(e) as BoxedError))?
                } else {
                    panic!()
                    // self.get_next_page(id).await
                }
                .videos
            })
        } else {
            Err(BackendError::from("Missing the playlist id.".to_owned()))
        }
    }

    // TODO: make this panic-safe
    async fn get_first_page(&mut self) -> Result<PlaylistPage, ReqwestError> {
        let result = self
            .client
            .get(&format!(
                "{}&playlistId={}&maxResults={}&key={}",
                YOUTUBE_API_URL,
                self.playlist_id.as_ref().unwrap(),
                self.items_per_page,
                self.api_key
            ))
            .send()
            .await?
            .json::<Value>()
            .await?;

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
                .into_iter()
                .map(|v| YouTubeVideo {
                    id: v["contentDetails"]["videoId"].as_str().unwrap().to_owned(),
                })
                .collect(),
        })
    }
}
