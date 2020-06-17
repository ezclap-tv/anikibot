use std::collections::HashMap;

#[derive(Clone)]
pub struct CommandData {
    pub usage: String,
    pub script: String,
}

pub struct Command {
    pub commands: Option<HashMap<String, Command>>,
    pub data: Option<CommandData>,
}
