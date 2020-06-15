extern crate config;
extern crate log;
extern crate pretty_env_logger;
extern crate tokio;
extern crate twitchchat;
extern crate chrono;
extern crate serde;
extern crate serde_json;

extern crate backend;

use backend::{Secrets, Bot, StreamElementsAPI, StreamElementsConfig};

use std::convert::Into;

use log::{error, info};
use twitchchat::{
    Dispatcher, RateLimit, Runner, Status,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());

    let secrets = Secrets::get();

    info!("Initializing the StreamElements API.");
    let api = StreamElementsAPI::new(
        StreamElementsConfig::with_token(secrets.stream_elements_jwt_token.clone()).unwrap(),
    )
    .finalize()
    .await
    .unwrap();

    info!("Initializing bot...");
    let bot = Bot::new(api, control.writer().clone(), control.clone()).run(dispatcher);

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
