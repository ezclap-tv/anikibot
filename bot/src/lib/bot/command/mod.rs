use super::util;
use crate::BackendError;
use serde::Deserialize;
use std::collections::HashMap;

pub(crate) fn load_lua<'a>(
    lua: &'a mlua::Lua,
    name: &str,
    source: &str,
) -> Result<mlua::Function<'a>, BackendError> {
    lua.load(source).into_function().map_err(|e| {
        BackendError::from(format!(
            "Failed to load the LUA script for `{}`: {}",
            name, e
        ))
    })
}

fn transform<'a>(
    lua: &'a mlua::Lua,
    commands: HashMap<String, CommandJSON>,
) -> HashMap<String, Command> {
    let mut transformed: HashMap<String, Command> = HashMap::new();
    for (name, command) in commands {
        let data: Option<CommandData> = match (command.usage, command.script) {
            (Some(usage), Some(script)) => Some(CommandData {
                usage,
                name: name.clone(),
                is_expensive: command.is_expensive.unwrap_or(false),
                path: script.clone(),
                script: load_lua(
                    &lua,
                    &name,
                    &util::load_file(&script)
                        .unwrap_or_else(|e| panic!("Failed to load file {}: {}", script, e)),
                )
                .unwrap_or_else(|e| panic!("Failed to load the script {}: {}", script, e)),
            }),
            _ => None,
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

pub fn load_commands<'a>(
    lua: &'a mlua::Lua,
    path: &str,
) -> Result<HashMap<String, Command<'a>>, BackendError> {
    Ok(transform(
        lua,
        util::parse_json(&util::load_file(path).map_err(|e| {
            BackendError::from(format!(
                "Failed to locate the commands json at {}: {}",
                path, e
            ))
        })?)?,
    ))
}

#[derive(Clone, Deserialize)]
struct CommandJSON {
    pub usage: Option<String>,
    pub is_expensive: Option<bool>,
    pub script: Option<String>,
    pub commands: Option<HashMap<String, CommandJSON>>,
}

#[derive(Clone)]
pub struct CommandData<'a> {
    pub usage: String,
    pub is_expensive: bool,
    pub path: String,
    pub name: String,
    pub script: mlua::Function<'a>,
}

pub struct Command<'a> {
    pub data: Option<CommandData<'a>>,
    pub commands: Option<HashMap<String, Command<'a>>>,
}
