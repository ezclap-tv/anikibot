use std::time::Duration;

use anyhow::Result;
use sqlx::{Connection, SqliteConnection};

mod config;
use config::Config;

fn init_logger() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    Ok(pretty_env_logger::try_init()?)
}

struct Bot {
    config: Config,
    db: sqlx::SqliteConnection,
    conn: twitch::Connection,
    script: script::Engine,
}

impl Bot {
    pub async fn init(config: Config) -> Result<Bot> {
        let mut db = sqlx::SqliteConnection::connect(&format!("sqlite:{}", config.database_name)).await?;
        sqlx::migrate!().run(&mut db).await?;
        let mut conn = twitch::connect(config.twitch()).await?;
        // TEMP: manage connected channels through db
        conn.sender.join("moscowwbish").await?;
        let script = script::Engine::init(script::Config {
            memory_limit: Some(config.worker_memory_limit),
        })?;
        conn.sender.privmsg("moscowwbish", "Connected").await?;

        Ok(Bot {
            config,
            db,
            conn,
            script,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        self.conn = twitch::connect(twitch::Config::default()).await?;
        // TEMP: manage connected channels through db
        self.conn.sender.join("moscowwbish").await?;
        self.conn.sender.privmsg("moscowwbish", "Connected").await?;

        Ok(())
    }

    pub fn reinit_lua(&mut self) -> Result<()> {
        self.script = script::Engine::init(script::Config {
            memory_limit: Some(self.config.worker_memory_limit),
        })?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    let mut bot = Bot::init(Config::init(&format!(
        "{}/Config.toml",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    )))
    .await?;

    bot.script.scope(|lua| {
        lua.globals().set(
            "sleep",
            lua.create_async_function(|_, secs: u64| async move {
                tokio::time::sleep(Duration::from_secs(secs)).await;
                Ok(())
            })?,
        )?;

        Ok(())
    })?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            },
            msg = bot.conn.reader.next() => match msg {
                Ok(msg) => {
                    match msg {
                    twitch::Message::Ping(ping) => {
                        log::info!("Got PING");
                        bot.conn.sender.pong(ping.arg()).await?;
                        log::info!("Sent PONG");
                    },
                    twitch::Message::Privmsg(message) => {
                        log::info!("#{} {} ({}): {}", message.channel(), message.user.name, message.user.id(), message.text());
                        if message.user.login() == "moscowwbish" || message.user.login() == "compileraddict" {
                            if let Some(code) = message.text().strip_prefix("!eval ") {
                                match bot.script.eval_async::<(), String>(code, ()).await {
                                    Ok(r) => {
                                        log::info!("[LUA] -> {}", r);
                                        bot.conn.sender.privmsg(message.channel(), &r).await?;
                                    },
                                    Err(e) => match e {
                                        script::Error::Memory(e) => {
                                            log::error!("[LUA] OUT OF MEMORY -> {}", e);
                                            bot.reinit_lua().unwrap();
                                        }
                                        _ => log::error!("[LUA] -> {}", &e)
                                    }
                                }
                            }
                        }
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
