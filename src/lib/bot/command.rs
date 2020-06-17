
use super::Bot;
use twitchchat::messages;

use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;

type ResponseFactory = for<'a> fn(
    &'a mut Bot,
    evt: &'a messages::Privmsg<'_>,
    Option<Vec<&'a str>>,
) -> Pin<Box<dyn Future<Output = String> + 'a>>;

#[derive(Clone)]
pub struct CommandData {
    /// Contains info about command usage
    pub help: String,
    /// Pointer to function with command logic
    /// This should eventually be replaced by a script
    pub factory: ResponseFactory,
}

pub struct Command {
    pub commands: Option<HashMap<String, Command>>,
    pub data: Option<CommandData>,
}