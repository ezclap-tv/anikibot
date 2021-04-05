use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use async_channel as mpmc;
use tokio::sync::Mutex;
use tokio::{runtime::Handle, sync::mpsc};

use crate::{config::Config, util};

#[derive(Clone, Debug)]
pub enum Instruction {
    Debug { what: String },
    LoadCommand { name: Arc<String>, code: Arc<String> },
    UnloadCommand { name: Arc<String> },
}

pub struct Command {
    pub source: twitch::Privmsg,
    pub name: String,
    pub args: String,
    pub privileged: bool,
}

impl Command {
    pub fn new(source: twitch::Privmsg, name: String, args: String, privileged: bool) -> Command {
        Command {
            source,
            name,
            args,
            privileged,
        }
    }
}

pub struct Worker {
    id: usize,
    config: Config,
    ctx: script::Context,
    inst_receiver: mpsc::Receiver<Instruction>,
    msg_receiver: Arc<mpmc::Receiver<Command>>,
    tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
}

unsafe impl Send for Worker {}

impl Worker {
    pub fn spawn(
        id: usize,
        config: Config,
        inst_receiver: mpsc::Receiver<Instruction>,
        msg_receiver: Arc<mpmc::Receiver<Command>>,
        tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
    ) -> JoinHandle<()> {
        let tokio_handle = Handle::current();
        thread::spawn(move || {
            tokio_handle.block_on(async move {
                Worker::new(id, config, inst_receiver, msg_receiver, tmi_sender)
                    .run()
                    .await;
            });
        })
    }

    pub fn new(
        id: usize,
        config: Config,
        inst_receiver: mpsc::Receiver<Instruction>,
        msg_receiver: Arc<mpmc::Receiver<Command>>,
        tmi_sender: Arc<Mutex<twitch::conn::Sender>>,
    ) -> Worker {
        let script_config = config.script();
        Worker {
            id,
            config,
            ctx: script::Context::init(script_config).expect("Failed to initialize worker script context"),
            inst_receiver,
            msg_receiver,
            tmi_sender,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                Some(inst) = self.inst_receiver.recv() => self.handle_inst(inst).await,
                Ok(message) = self.msg_receiver.recv() => self.handle_msg(message).await
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
            Instruction::LoadCommand { name, code } => {
                self.ctx.unload(&name);
                if let Err(err) = self.ctx.load((*name).clone(), &code) {
                    log::error!("[Worker #{}] Failed to load command {}: {}", self.id, name, err);
                }
            }
            Instruction::UnloadCommand { name } => self.ctx.unload(&name),
        }
    }

    pub async fn handle_msg(&mut self, command: Command) {
        let result = match &command.name[..] {
            "eval" if command.privileged => {
                self.ctx
                    .eval_async::<script::Variadic, String>(&command.args, script::Variadic::new())
                    .await
            }
            name if self.ctx.exists(name) => {
                self.ctx
                    .exec_async::<script::Variadic, String>(&command.name, util::parse_args(&command.args, true))
                    .await
            }
            _ => return,
        };
        match result {
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
                }
                _ => log::error!("[Worker #{}] -> {}", self.id, &e),
            },
        }
    }
}
