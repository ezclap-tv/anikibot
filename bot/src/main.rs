extern crate better_panic;
extern crate chrono;
extern crate config;
extern crate log;
extern crate pretty_env_logger;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate twitchchat;

extern crate backend;

use backend::{
    lua::init_globals, youtube::YouTubePlaylistAPI, Bot, Secrets, StreamElementsAPI,
    StreamElementsConfig,
};
use std::convert::Into;
use twitchchat::{Dispatcher, RateLimit, Runner, Status};

#[tokio::main]
async fn main() {
    better_panic::install();
    pretty_env_logger::init();

    log::info!("Creating a Lua instance.");
    let lua = mlua::Lua::new();

    let dispatcher = Dispatcher::new();
    let (runner, control) = Runner::new(
        dispatcher.clone(),
        RateLimit::full(1, std::time::Duration::from_secs(1)),
    );

    let secrets = Secrets::get();

    log::info!("Initializing the StreamElements API.");
    let mut thread_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();

    log::info!("Initializing bot...");
    let bot = {
        let mut builder = Bot::builder(control);
        if let Some(ref key) = secrets.stream_elements_jwt_token {
            let (api, handle) = StreamElementsAPI::with_config(
                StreamElementsConfig::with_token(key.to_owned()).unwrap(),
            )
            .start(tokio::runtime::Handle::current())
            .await
            .expect("Failed to start thread");

            thread_handles.push(handle);
            builder = builder.add_streamelements_api(api);
        }
        if let Some(ref key) = secrets.youtube_api_key {
            let (api, handle) = YouTubePlaylistAPI::with_api_key(key.to_owned())
                .start(tokio::runtime::Handle::current());
            thread_handles.push(handle);
            builder = builder.add_youtube_api(api);
        }

        builder.build(&lua)
    };
    init_globals(&lua, &bot);

    let bot_done = bot.run(&lua, dispatcher);

    log::info!("Connecting to twitch...");
    let conn = twitchchat::connect_tls(&secrets.into()).await.unwrap();

    let runner_done = runner.run(conn);

    tokio::select! {
        _ = bot_done => { log::info!("Bot stopped") },
        status = runner_done => {
            match status {
                Ok(Status::Canceled) => {
                    log::error!("Runner cancelled");
                },
                Ok(Status::Eof) => {
                    log::error!("Got EOF");
                },
                Ok(Status::Timeout) => {
                    log::error!("Timed out");
                },
                Err(err) => {
                    panic!(format!("{}", err));
                }
            }
        }
    }
}
