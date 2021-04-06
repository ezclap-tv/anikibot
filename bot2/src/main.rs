#![feature(str_split_once)]
mod config;
mod db;
mod util;
mod worker;

use std::{collections::HashMap, sync::Arc, thread::JoinHandle, time::Duration};

use anyhow::Result;
use async_channel as mpmc;
use config::Config;
use tokio::sync::{mpsc, Mutex};
use worker::Worker;

fn init_logger() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    Ok(alto_logger::init_term_logger()?)
}

struct Bot {
    config: Config,
    db: sqlx::SqliteConnection,
    // DB cache
    channels: HashMap<String, db::Channel>,
    commands: HashMap<String, db::Command>,

    inst_senders: Vec<mpsc::Sender<worker::Instruction>>,
    /// TMI Message Sender (wrapper over a TCP stream write half)
    tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
    /// Sender to Workers (for handling messages)
    msg_sender: mpmc::Sender<worker::Command>,
    /// TMI Message Reader (wrapper over a TCP stream read half)
    tmi_reader: twitch::conn::Reader,
    _workers: Vec<JoinHandle<()>>,
}

macro_rules! respond {
    ($self:ident, $channel:expr, $msg:literal) => {{
        $self.tmi_sender
            .lock()
            .await
            .privmsg($channel, $msg)
            .await?;
    }};
    ($self:ident, $channel:expr, $($arg:tt)*) => {{
        $self.tmi_sender
            .lock()
            .await
            .privmsg($channel, &format!($($arg)*))
            .await?;
    }}
}

macro_rules! broadcast {
    ($self:ident, $inst:expr) => {
        async {
            let inst = $inst;
            for sender in $self.inst_senders.iter() {
                sender.send(inst.clone()).await?;
            }
            anyhow::Result::<()>::Ok(())
        }
    };
}

impl Bot {
    pub async fn init(config: Config) -> Result<Bot> {
        // init db
        let mut db = db::connect(true).await?;

        // connect to twitch
        let (mut tmi_sender, tmi_reader) = twitch::connect(config.twitch()).await?.split();

        // join channels
        // main channel
        tmi_sender.join(&config.main_channel).await?;
        // persisted channels
        let mut channels = HashMap::new();
        for channel in db::Channel::all(&mut db).await? {
            if channel.joined == 1 {
                log::info!("Joining {}", channel.name);
                tmi_sender.join(&channel.name).await?;
            }
            channels.insert(channel.name.clone(), channel);
        }

        let tmi_sender = Arc::new(Mutex::new(tmi_sender));

        // multi-producer multi-consumer queue for incoming messages
        // these will be produced by the main message loop
        // and consumed by workers
        let (msg_sender, msg_receiver) = mpmc::bounded(config.concurrency);
        let msg_receiver = Arc::new(msg_receiver);

        // init worker threads
        // these senders are for broadcasting to all workers
        let mut inst_senders = Vec::with_capacity(config.concurrency);
        // this is for holding the worker thread join handles
        let mut workers = Vec::with_capacity(config.concurrency);
        for id in 0..config.concurrency {
            let (inst_sender, inst_receiver) = mpsc::channel(4);
            inst_senders.push(inst_sender);

            workers.push(Worker::spawn(
                id,
                config.clone(),
                inst_receiver,
                msg_receiver.clone(),
                tmi_sender.clone(),
            ))
        }

        tmi_sender.lock().await.privmsg("moscowwbish", "Connected").await?;

        let mut bot = Bot {
            config,
            db,
            commands: HashMap::new(),
            channels,
            inst_senders,
            tmi_sender,
            msg_sender,
            tmi_reader,
            _workers: workers,
        };

        // initialize commands from db
        for cmd in db::Command::all(&mut bot.db).await? {
            let (name, code) = (Arc::new(cmd.name().clone()), Arc::new(cmd.code.clone()));
            broadcast!(bot, worker::Instruction::LoadCommand { name, code }).await?;
            bot.commands.insert(cmd.name().clone(), cmd);
        }

        Ok(bot)
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let (mut tmi_sender, tmi_reader) = twitch::connect(twitch::Config::default()).await?.split();
        tmi_sender.join(&self.config.main_channel).await?;

        for channel in self.channels.values() {
            if channel.joined == 1 {
                tmi_sender.join(&channel.name).await?;
            }
        }

        *(self.tmi_sender.lock().await) = tmi_sender;
        self.tmi_reader = tmi_reader;

        Ok(())
    }

    pub fn is_privileged_user(&mut self, login: &str) -> bool {
        // TODO: configure this through db
        ["moscowwbish", "compileraddict"].contains(&login)
    }

    async fn create(&mut self, name: String, code: String) -> Result<&'static str> {
        match self.commands.get_mut(&name) {
            Some(command) => {
                // persist
                command.code = code.clone();
                command.save(&mut self.db).await?;

                // propagate to workers
                let (name, code) = (Arc::new(name), Arc::new(code));
                broadcast!(self, worker::Instruction::LoadCommand { name, code }).await?;
            }
            None => {
                // persist
                let mut command = db::Command::new(name.clone(), code.clone());
                command.save(&mut self.db).await?;
                self.commands.insert(command.name().clone(), command);

                // propagate to workers
                let (name, code) = (Arc::new(name), Arc::new(code));
                broadcast!(self, worker::Instruction::LoadCommand { name, code }).await?;
            }
        }

        Ok("Command saved")
    }

    async fn delete(&mut self, name: String) -> Result<&'static str> {
        match self.commands.remove(&name) {
            Some(mut command) => {
                command.delete(&mut self.db).await?;
                let name = Arc::new(name);
                broadcast!(self, worker::Instruction::UnloadCommand { name }).await?;
                Ok("Command deleted")
            }
            None => Ok("Command does not exist"),
        }
    }

    async fn join(&mut self, name: String, prefix: Option<String>) -> Result<&'static str> {
        if name == self.config.main_channel {
            Ok("Can't join main channel")
        } else {
            match self.channels.get_mut(&name) {
                Some(channel) if channel.joined == 1 => Ok("Channel already joined"),
                Some(channel) => {
                    channel.joined = 1;
                    if let Some(prefix) = prefix {
                        channel.prefix = prefix;
                    }
                    channel.save(&mut self.db).await?;
                    // TODO: actually check if bot joins the channel
                    self.tmi_sender.lock().await.join(&name).await?;
                    Ok("Channel joined successfully")
                }
                _ => {
                    let mut channel = db::Channel::new(name, prefix);
                    channel.save(&mut self.db).await?;
                    self.tmi_sender.lock().await.join(&channel.name).await?;
                    self.channels.insert(channel.name.clone(), channel);
                    Ok("Channel joined successfully")
                }
            }
        }
    }

    async fn leave(&mut self, which: &str) -> Result<Option<&'static str>> {
        if which == self.config.main_channel {
            Ok(Some("Can't leave the main channel"))
        } else {
            match self.channels.get_mut(which) {
                Some(channel) if channel.joined == 1 => {
                    channel.joined = 0;
                    channel.save(&mut self.db).await?;
                    // TODO: actually check if this is true
                    // for now it just leaves and doesn't care if it doesn't work
                    self.tmi_sender.lock().await.part(which).await?;
                    Ok(None)
                }
                _ => Ok(Some("Channel not joined")),
            }
        }
    }

    async fn prefix(&mut self, which: &str, prefix: String) -> Result<&'static str> {
        if which == self.config.main_channel {
            Ok("Can't change main channel prefix")
        } else {
            match self.channels.get_mut(which) {
                Some(channel) => {
                    channel.prefix = prefix;
                    channel.save(&mut self.db).await?;
                    Ok("Prefix changed")
                }
                None => Ok("Channel not joined"),
            }
        }
    }

    async fn handle_msg(&mut self, message: twitch::Privmsg) -> Result<()> {
        let cmd_prefix = match self.channels.get(message.channel()) {
            Some(v) => &v.prefix,
            None if message.channel() == self.config.main_channel => &self.config.main_channel_prefix,
            _ => return Ok(()),
        };
        if let Some((name, args)) = util::split_cmd(cmd_prefix, message.text()) {
            // TODO: yank impls of these commands somewhere else
            // so that they can be re-used with the bot REST API
            // - they should just be methods.
            match name {
                "ping" => {
                    respond!(self, message.channel(), "Pong!");
                }
                "create" if self.is_privileged_user(message.user.login()) => {
                    // !create <cmd name> <code>
                    let mut args = util::parse_args(args, false);
                    if args.len() > 1 {
                        let name = args.remove(0);
                        let code = args.join(" ");
                        let res = self.create(name, code).await?;
                        respond!(self, message.channel(), "{}", res)
                    } else {
                        respond!(self, message.channel(), "Usage: !create <name> <code>");
                    }
                }
                "delete" if self.is_privileged_user(message.user.login()) => {
                    // !delete <cmd name>
                    let mut args = util::parse_args(args, true);
                    if !args.is_empty() {
                        let name = args.remove(0);
                        let res = self.delete(name).await?;
                        respond!(self, message.channel(), "{}", res);
                    } else {
                        respond!(self, message.channel(), "Usage: !delete <name>");
                    }
                }
                "join" if self.is_privileged_user(message.user.login()) => {
                    // !join <channel> [prefix]
                    let mut args = util::parse_args(args, true);
                    if args.len() > 0 {
                        let name = args.remove(0);
                        let prefix = if args.len() > 0 { Some(args.remove(0)) } else { None };
                        let res = self.join(name, prefix).await?;
                        respond!(self, message.channel(), "{}", res);
                    } else {
                        respond!(self, message.channel(), "Usage: !join <channel> [prefix]");
                    }
                }
                "leave" if self.is_privileged_user(message.user.login()) => {
                    // !leave <channel>
                    // !leave this
                    let mut args = util::parse_args(args, true);
                    if !args.is_empty() {
                        let name = args.remove(0);
                        let which = if &name == "this" { message.channel() } else { &name };
                        let res = self.leave(which).await?;
                        if let Some(res) = res {
                            respond!(self, message.channel(), "{}", res);
                        }
                    } else {
                        respond!(self, message.channel(), "Usage: !leave <channel>");
                    }
                }
                "prefix" if self.is_privileged_user(message.user.login()) => {
                    // !prefix <channel> <prefix>
                    // !prefix this <prefix>
                    let mut args = util::parse_args(args, true);
                    if args.len() > 1 {
                        let name = args.remove(0);
                        let prefix = args.remove(0);
                        let which = if &name == "this" { message.channel() } else { &name };
                        let res = self.prefix(which, prefix).await?;
                        respond!(self, message.channel(), "{}", res);
                    } else {
                        respond!(self, message.channel(), "Usage: !prefix <channel> <prefix>");
                    }
                }
                _ => {
                    let name = name.to_string();
                    let args = args.to_string();
                    let privileged = self.is_privileged_user(message.user.login());
                    self.msg_sender
                        .send(worker::Command::new(message, name, args, privileged))
                        .await?
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break Ok(());
                },
                msg = self.tmi_reader.next() => match msg {
                    Ok(msg) => {
                        match msg {
                        twitch::Message::Ping(ping) => {
                            self.tmi_sender.lock().await.pong(ping.arg()).await?;
                        },
                        twitch::Message::Privmsg(message) => self.handle_msg(message).await?,
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
