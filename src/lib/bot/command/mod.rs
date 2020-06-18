

use std::collections::HashMap;
use std::sync::Mutex;
use serde::Deserialize;
use super::util;

#[derive(Deserialize)]
struct CommandJSON {
    pub usage: Option<String>,
    pub script: Option<String>,
    pub commands: Option<HashMap<String, CommandJSON>>,
}

lazy_static! {
    static ref LUA_CONTEXT: Mutex<mlua::Lua> = { 
        Mutex::new(mlua::Lua::new())
    };
}

fn load_lua(name: &str, script: &str) -> mlua::Function<'static> {
    LUA_CONTEXT.load(script).into_function()
        .expect(&format!("Failed to load LUA script {}", name))
}

fn transform(commands: HashMap<String, CommandJSON>) -> HashMap<String, Command> {
    let mut transformed: HashMap<String, Command> = HashMap::new();
    for (name, command) in commands {
        let data: Option<CommandData> = if command.usage.is_some() && command.script.is_some() {
            Some(CommandData {
                usage: command.usage.unwrap(),
                script: load_lua(&name, &util::load_file(&command.script.unwrap()))
            })
        } else {
            None
        };

        transformed.insert(name, Command {
            data,
            commands: if command.commands.is_some() {
                Some(transform(command.commands.unwrap()))
            } else {
                None
            }
        });
    }
    transformed
}

pub fn load_commands(path: &str) -> HashMap<String, Command> {
    transform(util::parse_json(&util::load_file(path)))
}

#[derive(Clone)]
pub struct CommandData {
    pub usage: String,
    pub script: mlua::Function<'static>,
}

pub struct Command {
    pub data: Option<CommandData>,
    pub commands: Option<HashMap<String, Command>>,
}
