use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

pub struct Worker {
    id: usize,
    ctx: script::Context,
    sender: Arc<Mutex<twitch::conn::Sender>>,
}

unsafe impl Send for Worker {}

impl Worker {
    pub fn new(id: usize, memory_limit: usize, sender: Arc<Mutex<twitch::conn::Sender>>) -> Worker {
        Worker {
            id,
            ctx: script::Context::init(script::Config {
                memory_limit: Some(memory_limit),
            })
            .expect("Failed to initialize worker"),
            sender,
        }
    }

    pub fn reset(&mut self) {
        log::info!("[LUA #{}] Reset", self.id);
        self.ctx = script::Context::init(self.ctx.config).expect("Failed to initialize worker");
    }

    pub async fn handle(&mut self, message: twitch::Privmsg) -> Result<()> {
        if message.user.login() == "moscowwbish" || message.user.login() == "compileraddict" {
            let text = message.text();
            let channel = message.channel();
            if let Some(code) = text.strip_prefix("!eval ") {
                match self.ctx.eval_async::<(), String>(code, ()).await {
                    Ok(r) => {
                        log::info!("[LUA #{}] -> {}", self.id, r);
                        let mut sender = self.sender.lock().await;
                        sender.privmsg(channel, &r).await?;
                    }
                    Err(e) => match e {
                        script::Error::Memory(_) => {
                            log::error!("[LUA #{}] ran out of memory", self.id);
                            self.reset();
                        }
                        _ => log::error!("[LUA #{}] -> {}", self.id, &e),
                    },
                }
            }
        }
        Ok(())
    }
}
