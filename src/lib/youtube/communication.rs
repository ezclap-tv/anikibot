use tokio::sync::{mpsc, oneshot};

use crate::BackendError;

use super::api::YouTubePlaylistAPI;
use super::config::YouTubeAPIConfig;
use super::data::Videos;

/// The type that is send back by the API thread.
pub type APIResponse = Result<APIResponseMessage, BackendError>;
/// A tuple of (sender, thread handle) returned by `spawn_api_thread`.
pub type APIHandle = (RequestSender, std::thread::JoinHandle<()>);
/// The request `Sender` channel type.
pub type RequestSender = mpsc::UnboundedSender<APIRequestMessage>;
/// The request `Sender` channel type.
pub type ResponseSender = oneshot::Sender<APIResponse>;

/// Indicates the kind of the API request to be made by the API thread.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum APIRequestKind {
    // Playlist configuration
    Playlist_Set { id: String },
    Playlist_SetPageSize(usize),
    Playlist_Configure { id: String, page_size: usize },
    Playlist_Get,
    Playlist_GetPageSize,
    Playlist_GetConfig,

    // Playlist API
    Playlist_GetPlaylistVideos,
}

/// A message sent to the API thread.
#[derive(Debug)]
pub struct APIRequestMessage {
    /// The the kind of the API request to be made by the API thread.
    pub(crate) kind: APIRequestKind,
    /// The output channel used to receive the API call result.
    pub(crate) output: ResponseSender,
}

/// A response that contains the result of the API call if it succeeds.
#[derive(Debug)]
pub enum APIResponseMessage {
    /// An empty result indicating that the requested operation have been completed successfully.
    Done,
    /// A numeric result value.
    Number(usize),
    /// A string result value.
    Str(String),
    /// A `serde_json::Value` object containing the JSON returned by the server.
    Json(serde_json::Value),
    /// The current publicly accessible configuration of the API thread.
    Config(YouTubeAPIConfig),
    /// The videos in the current playlist page.
    Videos(Videos),
}

impl APIResponseMessage {
    #[inline]
    pub fn into_videos(self) -> Result<Videos, Self> {
        match self {
            Self::Videos(videos) => Ok(videos),
            _ => Err(self),
        }
    }
}

/// Spawns a YouTube API thread.
/// Returns the sender part of the channel and the thread handle.
pub(crate) fn spawn_api_thread(
    mut api: YouTubePlaylistAPI,
    runtime: tokio::runtime::Handle,
) -> (RequestSender, std::thread::JoinHandle<()>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<APIRequestMessage>();

    log::trace!("Spawning the YouTube API thread...");

    let handle = std::thread::spawn(move || {
        runtime.block_on(async move {
            log::trace!("Successfully spawned the YouTube API thread.");

            while let Some(msg) = rx.recv().await {
                log::trace!("Received a YouTube API request: {:#?}", msg.kind);

                let result = match msg.kind {
                    APIRequestKind::Playlist_Set { id } => {
                        api.set_playlist(id);
                        Ok(APIResponseMessage::Done)
                    }
                    APIRequestKind::Playlist_SetPageSize(size) => {
                        api.page_size(size);
                        Ok(APIResponseMessage::Done)
                    }
                    APIRequestKind::Playlist_Configure { id, page_size } => {
                        api.set_playlist(id);
                        api.page_size(page_size);
                        Ok(APIResponseMessage::Done)
                    }
                    APIRequestKind::Playlist_Get => Ok(APIResponseMessage::Str(
                        api.current_playlist()
                            .map(String::from)
                            .unwrap_or_else(String::new),
                    )),
                    APIRequestKind::Playlist_GetPageSize => {
                        Ok(APIResponseMessage::Number(api.items_per_page))
                    }
                    APIRequestKind::Playlist_GetConfig => {
                        Ok(APIResponseMessage::Config(api.get_config()))
                    }
                    APIRequestKind::Playlist_GetPlaylistVideos => {
                        yt_resp_videos!(api.get_playlist_videos().await)
                    }
                };
                msg.output.send(result).unwrap();
            }

            log::trace!("Terminating the YouTube API thread...")
        });

        log::trace!("Successfully terminated the YouTube API thread.");
    });

    (tx, handle)
}
