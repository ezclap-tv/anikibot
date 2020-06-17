use crate::BackendError;
use tokio::sync::{mpsc, oneshot};

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
    // Channel API
    Channel_Me,
    Channel_MyId,
    Channel_Chan {
        name: String,
    },
    Channel_Id {
        name: String,
    },
    // SongRequest API
    SongReq_Settings,
    SongReq_PublicSettings {
        channel_id: String,
    },
    SongReq_CurrentSong,
    SongReq_CurrentSongTitle,
    SongReq_QueueSong {
        song_url: String,
    },
    SongReq_QueueSongInChannel {
        channel_id: String,
        song_url: String,
    },
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
    /// A `serde_json::Value` object containing the JSON returned by the server.
    Json(serde_json::Value),
    /// A string result value.
    Str(String),
}

/// Spawns a StreamElements API thread.
/// Returns the sender part of the channel and the thread handle.
pub(crate) fn spawn_api_thread(
    api: crate::StreamElementsAPI,
    runtime: tokio::runtime::Handle,
) -> (RequestSender, std::thread::JoinHandle<()>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<APIRequestMessage>();

    log::trace!("Spawning the StreamElements API thread...");

    let handle = std::thread::spawn(move || {
        runtime.block_on(async move {
            log::trace!("Successfully spawned the StreamElements API thread.");

            while let Some(msg) = rx.recv().await {
                log::trace!("Received a StreamElements API request: {:#?}", msg.kind);

                let result = match msg.kind {
                    // CHannel API
                    APIRequestKind::Channel_Me => resp_json!(api.channels().me().await),
                    APIRequestKind::Channel_MyId => resp_str!(api.channels().my_id().await),
                    APIRequestKind::Channel_Chan { name } => {
                        resp_json!(api.channels().channel(&name).await)
                    }
                    APIRequestKind::Channel_Id { name } => {
                        resp_str!(api.channels().channel_id(&name).await)
                    }
                    // SongRequest API
                    APIRequestKind::SongReq_Settings => {
                        resp_json!(api.song_requests().get_settings().await)
                    }
                    APIRequestKind::SongReq_PublicSettings { channel_id } => {
                        resp_json!(api.song_requests().get_public_settings(&channel_id).await)
                    }
                    APIRequestKind::SongReq_CurrentSong => {
                        resp_json!(api.song_requests().current_song().await)
                    }
                    APIRequestKind::SongReq_CurrentSongTitle => {
                        resp_str!(api.song_requests().current_song_title().await)
                    }
                    APIRequestKind::SongReq_QueueSong { song_url } => {
                        resp_json!(api.song_requests().queue_song(&song_url).await)
                    }
                    APIRequestKind::SongReq_QueueSongInChannel {
                        channel_id,
                        song_url,
                    } => resp_json!(
                        api.song_requests()
                            .queue_song_in_channel(&channel_id, &song_url)
                            .await
                    ),
                };
                msg.output.send(result).unwrap();
            }

            log::trace!("Terminating the StreamElements API thread...")
        });

        log::trace!("Successfully terminated the StreamElements API thread.");
    });

    (tx, handle)
}
