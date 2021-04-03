mod config;
mod util;
mod worker;

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use config::Config;
use sqlx::{Connection, SqliteConnection};
use tokio::sync::Mutex;
use util::Pool;
use worker::Worker;

fn init_logger() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    Ok(pretty_env_logger::try_init()?)
}

struct Bot {
    config: Config,
    db: sqlx::SqliteConnection,
    sender: Arc<Mutex<twitch::conn::Sender>>,
    reader: twitch::conn::Reader,
    worker_pool: Pool<Worker>,
}

impl Bot {
    pub async fn init(config: Config) -> Result<Bot> {
        // init db
        let mut db = sqlx::SqliteConnection::connect(&format!("sqlite:{}", config.database_name)).await?;
        sqlx::migrate!().run(&mut db).await?;

        // connect to twitch
        let (mut sender, reader) = twitch::connect(config.twitch()).await?.split();
        // TEMP: manage connected channels through db
        sender.join("moscowwbish").await?;
        let sender = Arc::new(Mutex::new(sender));

        // init worker pool
        let mut id = 0usize;
        let worker_pool = Pool::new(config.concurrency, || {
            Worker::new(
                {
                    id += 1;
                    id
                },
                config.worker_memory_limit,
                sender.clone(),
            )
        });
        sender.lock().await.privmsg("moscowwbish", "Connected").await?;

        Ok(Bot {
            config,
            db,
            sender,
            reader,
            worker_pool,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let (mut sender, receiver) = twitch::connect(twitch::Config::default()).await?.split();
        // TEMP: manage connected channels through db
        sender.join("moscowwbish").await?;
        *(self.sender.lock().await) = sender;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    let mut bot = Bot::init(Config::init(&format!(
        "{}/Config.toml",
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into())
    )))
    .await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            },
            msg = bot.reader.next() => match msg {
                Ok(msg) => {
                    match msg {
                    twitch::Message::Ping(ping) => {
                        log::info!("Got PING");
                        bot.sender.lock().await.pong(ping.arg()).await?;
                        log::info!("Sent PONG");
                    },
                    twitch::Message::Privmsg(message) => {
                        tokio::spawn({
                            let worker = bot.worker_pool.get().await;
                            async move {
                                worker.handle(message)
                            }
                        });
                    },
                    other => log::info!("{:?}", other)
                }},
                Err(err) => match err {
                    twitch::conn::Error::StreamClosed => {
                        log::info!("Disconnected, attempting to reconnect...");
                        let mut success = false;
                        let mut attempts = 0;
                        while attempts < 10 {
                            match bot.reconnect().await {
                                Ok(_) => { success = true; break; },
                                Err(err) => log::error!("Failed to reconnect after attemp #{} ({}), retrying...", attempts, err)
                            }
                            tokio::time::sleep(Duration::from_secs(attempts * 3)).await;
                            attempts += 1;
                        }
                        if !success {
                            panic!("Failed to reconnect");
                        }
                    },
                    err => panic!("{}", err)
                }
            }
        }
    }

    Ok(())
}
