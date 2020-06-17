use crate::{BackendError, BoxedError};
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
    Channel_Chan { name: String },
    Channel_Id { name: String },
    // SongRequest API
    SongReq_Settings,
    SongReq_PublicSettings { channel_id: String },
    SongReq_CurrentSong,
    SongReq_CurrentSongTitle,
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
) -> (
    RequestSender,
    std::thread::JoinHandle<()>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<APIRequestMessage>();

    log::trace!("Spawning the StreamElements API thread...");

    let handle = std::thread::spawn(move || {
        runtime.block_on(async move {
            log::trace!("Successfully spawned the StreamElements API thread.");
    
            while let Some(msg) = rx.recv().await {
                log::trace!("Received a StreamElements API request: {:#?}", msg.kind);
    
                let result = match msg.kind {
                    APIRequestKind::Channel_MyId => api
                        .channels()
                        .my_id()
                        .await
                        .map(|res| APIResponseMessage::Str(res))
                        .map_err(|e| {
                            log::error!("Caught an error while processing a StreamElements API request: {:#?}", e);
                            BackendError::from(Box::new(e) as BoxedError)
                        }),
                    rest => unimplemented!("API method {:?} is not implemented", rest),
                };
                msg.output.send(result).unwrap();
            }

            log::trace!("Terminating the StreamElements API thread...")
        });

        log::trace!("Successfully terminated the StreamElements API thread.");
    });

    (tx, handle)
}
