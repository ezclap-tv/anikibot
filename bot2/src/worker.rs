use std::sync::Arc;
use std::{convert::TryFrom, iter::FromIterator};

use anyhow::Result;
use async_channel as mpmc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::config::Config;

#[derive(Clone, Debug)]
pub enum Instruction {
    Debug { what: String },
    CreateCommand { name: Arc<String>, code: Arc<String> },
    DeleteCommand { name: Arc<String> },
    EditCommand { name: Arc<String>, code: Arc<String> },
}

pub struct Command {
    pub source: twitch::Privmsg,
    pub name: String,
    pub args: script::Variadic,
}

impl Command {
    pub fn parse(source: twitch::Privmsg) -> Option<Command> {
        if let Some(cmd) = source.text().strip_prefix('!') {
            let (name, args) = match cmd.split_once(' ') {
                Some((name, args)) => (name.to_string(), args.split(' ').map(String::from).collect()),
                None => (cmd.to_string(), script::Variadic::new()),
            };

            Some(Command { source, name, args })
        } else {
            None
        }
    }
}

pub struct Worker {
    id: usize,
    config: Config,
    /* db: sqlx::SqliteConnection, */
    ctx: script::Context,
    inst_receiver: mpsc::Receiver<Instruction>,
    tmi_receiver: Arc<mpmc::Receiver<Command>>,
    tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
}

unsafe impl Send for Worker {}

impl Worker {
    pub fn new(
        id: usize,
        config: Config,
        /* db: sqlx::SqliteConnection, */
        inst_receiver: mpsc::Receiver<Instruction>,
        tmi_receiver: Arc<mpmc::Receiver<Command>>,
        tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
    ) -> Worker {
        let script_config = config.script();
        Worker {
            id,
            config,
            /* db, */
            ctx: script::Context::init(script_config).expect("Failed to initialize worker script context"),
            inst_receiver,
            tmi_receiver,
            tmi_sender,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                Some(inst) = self.inst_receiver.recv() => self.handle_inst(inst).await,
                Ok(message) = self.tmi_receiver.recv() => self.handle_msg(message).await
            }
        }
    }

    pub fn reset(&mut self) {
        log::info!("[Worker #{}] Reset", self.id);
        self.ctx.reset();
    }

    pub async fn handle_inst(&mut self, inst: Instruction) {
        match inst {
            Instruction::Debug { what } => log::info!("[Worker #{}] Debug instruction: {}", self.id, what),
            Instruction::CreateCommand { name, code } => {
                self.ctx.unload(&name);
                if let Err(err) = self.ctx.load((*name).clone(), &code) {
                    log::error!("[Worker #{}] Failed to load command {}: {}", self.id, name, err);
                }
            }
            Instruction::DeleteCommand { name } => self.ctx.unload(&name),
            Instruction::EditCommand { name, code } => {
                self.ctx.unload(&name);
                if let Err(err) = self.ctx.load((*name).clone(), &code) {
                    log::error!("[Worker #{}] Failed to load command {}: {}", self.id, name, err);
                }
            }
        }
    }

    pub async fn handle_msg(&mut self, command: Command) {
        match &command.name[..] {
            "eval" if !command.args.is_empty() => {
                match self.ctx.eval_async::<(), String>(&command.args.join(" "), ()).await {
                    Ok(r) => {
                        log::info!("[Worker #{}] -> {}", self.id, r);
                        if let Err(err) = self.tmi_sender.lock().await.privmsg(command.source.channel(), &r).await {
                            // TODO: may need to properly handle some errors
                            log::error!("[Worker #{}] Error while writing to TMI: {}", self.id, err);
                        }
                    }
                    Err(e) => match e {
                        script::Error::Memory(_) => {
                            log::error!("[Worker #{}] Ran out of memory", self.id);
                            self.reset();
                            return;
                        }
                        _ => {
                            log::error!("[Worker #{}] -> {}", self.id, &e);
                            return;
                        }
                    },
                }
            }
            name if self.ctx.exists(name) => {
                match self
                    .ctx
                    .exec_async::<script::Variadic, String>(name, command.args)
                    .await
                {
                    Ok(r) => {
                        log::info!("[Worker #{}] -> {}", self.id, r);
                        if let Err(err) = self.tmi_sender.lock().await.privmsg(command.source.channel(), &r).await {
                            // TODO: may need to properly handle some errors
                            log::error!("[Worker #{}] Error while writing to TMI: {}", self.id, err);
                        }
                    }
                    Err(e) => match e {
                        script::Error::Memory(_) => {
                            log::error!("[Worker #{}] Ran out of memory", self.id);
                            self.reset();
                            return;
                        }
                        _ => {
                            log::error!("[Worker #{}] -> {}", self.id, &e);
                            return;
                        }
                    },
                }
            }
            _ => (),
        }
    }
}
