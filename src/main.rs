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
    let (runner, control) = Runner::new(dispatcher.clone(), RateLimit::default());

    let secrets = Secrets::get();

    info!("Initializing the StreamElements API.");
    let mut thread_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();

    info!("Initializing bot...");
    let bot =  {
        let mut builder = Bot::builder(control);
        if let Some(ref key) = secrets.stream_elements_jwt_token {
            let (api, handle) = StreamElementsAPI::new(
                StreamElementsConfig::with_token(key.to_owned()).unwrap(),
            )
            .start(tokio::runtime::Handle::current())
            .await
            .expect("Failed to start thread");

            thread_handles.push(handle);
            builder = builder.add_streamelements_api(api);
        }
        if let Some(ref key) = secrets.youtube_api_key {
            builder = builder.add_youtube_api(YouTubePlaylistAPI::new(key.to_owned()));
        }
        
        builder.build()
    };
    let bot_done = bot.run(dispatcher);

    info!("Connecting to twitch...");
    let conn = twitchchat::connect_tls(&secrets.into()).await.unwrap();

    let runner_done = runner.run(conn);

    tokio::select! {
        _ = bot_done => { info!("Bot stopped") },
        status = runner_done => {
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
