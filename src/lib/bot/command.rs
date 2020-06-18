

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize)]
struct CommandJSON {
    pub usage: Option<String>,
    pub script: Option<String>,
    pub commands: Option<HashMap<String, CommandJSON>>,
}

fn load_file(path: &str) -> String {
    let file = fs::File::open(path)
        .expect(&format!("Could not open file {}", path));
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)
        .expect(&format!("Failed to read file {}", path));

    contents
}

fn parse_json<'a, R>(json: &'a str) -> R 
where
    R: serde::Deserialize<'a>
{
    match from_str(&json) {
        Ok(json) => json,
        Err(e) => {
            panic!("Failed to read \"commands.json\": {}", e);
        }
    }
}

fn transform(commands: HashMap<String, CommandJSON>) -> HashMap<String, Command> {
    let mut transformed: HashMap<String, Command> = HashMap::new();
    for (name, command) in commands {
        transformed.insert(name, Command {
            usage: command.usage,
            script: if command.script.is_some() {
                Some(load_file(&command.script.unwrap()))
            } else {
                None
            },
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
    transform(parse_json(&load_file(path)))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Command {
    pub usage: Option<String>,
    pub script: Option<String>,
    pub commands: Option<HashMap<String, Command>>,
}
