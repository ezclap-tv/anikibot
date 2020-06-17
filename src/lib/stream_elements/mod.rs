pub mod api;
pub mod channels;
pub mod config;
pub mod song_requests;

use crate::{BackendError, BoxedError};
use tokio::sync::{mpsc, oneshot};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum APIRequestKind {
    // XXX: Maybe move into its own enum?
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

#[derive(Debug)]
pub enum APIRequestResponse {
    Json(serde_json::Value),
    Str(String),
}

#[derive(Debug)]
pub struct APIRequestMessage {
    pub kind: APIRequestKind,
    pub output: oneshot::Sender<Result<APIRequestResponse, BackendError>>,
}

fn spawn_api_thread(
    api: crate::StreamElementsAPI,
    runtime: tokio::runtime::Handle,
) -> (
    mpsc::UnboundedSender<APIRequestMessage>,
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
                        .map(|res| APIRequestResponse::Str(res))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_thread() {
        pretty_env_logger::init();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let handle = runtime.handle().clone();
        runtime.block_on(api_main(handle))
    }

    async fn api_main(runtime: tokio::runtime::Handle) {
        let token = crate::Secrets::get().stream_elements_jwt_token;
        let api =
            crate::StreamElementsAPI::new(crate::StreamElementsConfig::with_token(token).unwrap())
                .finalize()
                .await
                .unwrap();
        let (tx, _handle) = spawn_api_thread(api, runtime);

        let (to, ro) = oneshot::channel();
        tx.send(APIRequestMessage {
            kind: APIRequestKind::Channel_MyId,
            output: to,
        })
        .unwrap();

        log::trace!("Received {:#?} from the API", ro.await.unwrap());
    }
}
