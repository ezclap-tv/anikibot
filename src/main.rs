extern crate chrono;
extern crate config;
extern crate log;
extern crate pretty_env_logger;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate twitchchat;

extern crate backend;

use backend::{youtube::YouTubePlaylistAPI, Bot, Secrets, StreamElementsAPI, StreamElementsConfig};

use std::convert::Into;

use log::{error, info};
use twitchchat::{Dispatcher, RateLimit, Runner, Status};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());

    let secrets = Secrets::get();

    info!("Initializing the StreamElements API.");
    let thread_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();

    info!("Initializing bot...");
    let builder = Bot::builder(control);
    if let Some(ref key) = secrets.stream_elements_jwt_token {
        let (api, handle) = StreamElementsAPI::new(
            StreamElementsConfig::with_token(key.to_owned()).unwrap(),
        )
        .start(tokio::runtime::Handle::current())
        .await
        .expect("Failed to start thread");

        thread_handles.push(handle);
        builder.add_streamelements_api(api);
    }
    if let Some(ref key) = secrets.youtube_api_key {
        builder.add_youtube_api(YouTubePlaylistAPI::new(key.to_owned()));
    }
    let bot = if let Some(ref key) = secrets.youtube_api_key {
        Bot::with_youtube_api(
            api,
            YouTubePlaylistAPI::new(key.to_owned()),
            control.writer().clone(),
            control.clone(),
        )
        .run(dispatcher)
    } else {
        Bot::new(api, control.writer().clone(), control.clone()).run(dispatcher)
    };

    info!("Connecting to twitch...");
    let conn = twitchchat::connect_tls(&secrets.into()).await.unwrap();

    let done = runner.run(conn);

    tokio::select! {
        _ = bot => { info!("Bot stopped") },
        status = done => {
            match status {
                Ok(Status::Canceled) => {
                    error!("Runner cancelled");
                },
                Ok(Status::Eof) => {
                    error!("Got EOF");
                },
                Ok(Status::Timeout) => {
                    error!("Timed out");
                },
                Err(err) => {
                    panic!(format!("{}", err));
                }
            }
        }
    }
}
