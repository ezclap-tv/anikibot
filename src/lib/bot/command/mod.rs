use super::util;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct CommandJSON {
    pub usage: Option<String>,
    pub script: Option<String>,
    pub commands: Option<HashMap<String, CommandJSON>>,
}

fn load_lua<'a>(lua: &'a mlua::Lua, name: &str, script: &str) -> mlua::Function<'a> {
    lua.load(script)
        .into_function()
        .expect(&format!("Failed to load LUA script {}", name))
}

fn transform<'a>(
    lua: &'a mlua::Lua,
    commands: HashMap<String, CommandJSON>,
) -> HashMap<String, Command> {
    let mut transformed: HashMap<String, Command> = HashMap::new();
    for (name, command) in commands {
        let data: Option<CommandData> = if command.usage.is_some() && command.script.is_some() {
            Some(CommandData {
                usage: command.usage.unwrap(),
                script: load_lua(&lua, &name, &util::load_file(&command.script.unwrap())),
            })
        } else {
            None
        };

        transformed.insert(
            name,
            Command {
                data,
                commands: if command.commands.is_some() {
                    Some(transform(&lua, command.commands.unwrap()))
                } else {
                    None
                },
            },
        );
    }
    transformed
}

pub fn load_commands<'a>(lua: &'a mlua::Lua, path: &str) -> HashMap<String, Command<'a>> {
    transform(lua, util::parse_json(&util::load_file(path)))
}

#[derive(Clone)]
pub struct CommandData<'a> {
    pub usage: String,
    pub script: mlua::Function<'a>,
}

pub struct Command<'a> {
    pub data: Option<CommandData<'a>>,
    pub commands: Option<HashMap<String, Command<'a>>>,
}
