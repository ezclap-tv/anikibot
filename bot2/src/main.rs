#![feature(str_split_once)]
mod config;
mod worker;

const BOT_DB_NAME: &str = "sqlite:bot.db";

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use async_channel as mpmc;
use config::Config;
use sqlx::Connection;
use tokio::sync::{mpsc, Mutex};
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
    inst_senders: Vec<mpsc::Sender<worker::Instruction>>,
    /// TMI Message Sender (wrapper over a TCP stream write half)
    tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
    /// Sender to Workers (for handling messages)
    msg_sender: mpmc::Sender<worker::Command>,
    /// TMI Message Reader (wrapper over a TCP stream read half)
    tmi_reader: twitch::conn::Reader,
    _workers: Vec<JoinHandle<()>>,
}

impl Bot {
    pub async fn init(config: Config) -> Result<Bot> {
        // init db
        let mut db: sqlx::SqliteConnection = sqlx::SqliteConnection::connect(BOT_DB_NAME).await?;
        sqlx::migrate!().run(&mut db).await?;

        // connect to twitch
        let (mut tmi_sender, tmi_reader) = twitch::connect(config.twitch()).await?.split();
        // TEMP: manage connected channels through db
        tmi_sender.join("moscowwbish").await?;
        let tmi_sender = Arc::new(Mutex::new(tmi_sender));

        let (msg_sender, msg_receiver) = mpmc::bounded(config.concurrency);
        let msg_receiver = Arc::new(msg_receiver);

        // init worker threads
        let mut inst_senders = Vec::with_capacity(config.concurrency);
        let mut workers = Vec::with_capacity(config.concurrency);
        for id in 0..config.concurrency {
            let (inst_sender, inst_receiver) = mpsc::channel(4);
            inst_senders.push(inst_sender);

            // TODO: is db inside worker really necessary?
            /* let db = sqlx::SqliteConnection::connect(BOT_DB_NAME).await?; */
            let handle = tokio::runtime::Handle::current();
            let msg_receiver_clone = msg_receiver.clone();
            let tmi_sender_clone = tmi_sender.clone();
            let config_clone = config.clone();
            workers.push(thread::spawn(move || {
                handle.block_on(async move {
                    Worker::new(
                        id,
                        config_clone,
                        /* db, */
                        inst_receiver,
                        msg_receiver_clone,
                        tmi_sender_clone,
                    )
                    .run()
                    .await;
                })
            }))
        }

        tmi_sender.lock().await.privmsg("moscowwbish", "Connected").await?;

        Ok(Bot {
            config,
            db,
            inst_senders,
            tmi_sender,
            msg_sender,
            tmi_reader,
            _workers: workers,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let (mut sender, reader) = twitch::connect(twitch::Config::default()).await?.split();
        // TEMP: manage connected channels through db
        sender.join("moscowwbish").await?;
        *(self.tmi_sender.lock().await) = sender;
        self.tmi_reader = reader;

        Ok(())
    }

    pub async fn worker_broadcast(&mut self, inst: worker::Instruction) -> Result<()> {
        for sender in self.inst_senders.iter() {
            sender.send(inst.clone()).await?;
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // initialize commands from db
        #[derive(sqlx::FromRow)]
        struct Command {
            name: String,
            code: String,
        }
        for cmd in sqlx::query_as::<sqlx::Sqlite, Command>("SELECT name, code FROM command")
            .fetch_all(&mut self.db)
            .await?
        {
            let (name, code) = (Arc::new(cmd.name), Arc::new(cmd.code));
            self.worker_broadcast(worker::Instruction::CreateCommand { name, code })
                .await?;
        }
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break Ok(());
                },
                msg = self.tmi_reader.next() => match msg {
                    Ok(msg) => {
                        match msg {
                        twitch::Message::Ping(ping) => {
                            log::info!("Got PING");
                            self.tmi_sender.lock().await.pong(ping.arg()).await?;
                            log::info!("Sent PONG");
                        },
                        twitch::Message::Privmsg(message) => {
                            if let Some(cmd) = worker::Command::parse(message) {
                                // TODO: configure this through db
                                if cmd.source.user.login() == "moscowwbish" || cmd.source.user.login() == "compileraddict" {
                                    // TODO: so many allocations, remove pls
                                    match &cmd.name[..] {
                                        "test" => self.worker_broadcast(worker::Instruction::Debug { what: "Hello".into() }).await?,
                                        "create" => {
                                            if cmd.args.len() > 2 {
                                                let (name, code) = (cmd.args[0].clone(), cmd.args[1..].join(" "));
                                                log::info!("Create command '{}': \n{}", name, code);
                                                let (name, code) = (Arc::new(name), Arc::new(code));
                                                self.worker_broadcast(worker::Instruction::CreateCommand { name, code }).await?;
                                            } else {
                                                self.tmi_sender.lock().await.privmsg(cmd.source.channel(), "Usage: !create <name> <code>").await?;
                                            }
                                        },
                                        "delete" => {
                                            if !cmd.args.is_empty() {
                                                let name = Arc::new(cmd.args[0].clone());
                                                log::info!("Delete command '{}'", *name);
                                                self.worker_broadcast(worker::Instruction::DeleteCommand { name }).await?;
                                            } else {
                                                self.tmi_sender.lock().await.privmsg(cmd.source.channel(), "Usage: !delete <name>").await?;
                                            }
                                        },
                                        _ => self.msg_sender.send(cmd).await?
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
                                match self.reconnect().await {
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
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    Bot::init(Config::init(&format!(
        "{}/Config.toml",
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into())
    )))
    .await?
    .run()
    .await?;

    Ok(())
}
