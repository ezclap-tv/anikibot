use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use twitchchat::messages;

use super::Bot;

#[derive(Clone)]
pub struct CommandData {
    pub usage: String,
    pub script: String,
}

pub struct Command {
    pub commands: Option<HashMap<String, Command>>,
    pub data: Option<CommandData>,
}
