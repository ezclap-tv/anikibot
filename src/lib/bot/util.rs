use super::command::{Command, CommandData};

use std::collections::HashMap;

pub fn strip_prefix<'a>(str: &'a str, prefix: &str) -> &'a str {
    if !str.starts_with(prefix) {
        &str[..]
    } else {
        &str[prefix.len()..str.len()]
    }
}

pub fn duration_format(duration: chrono::Duration) -> String {
    let mut output = String::from("");

    let days = duration.num_days();
    if days > 0 {
        output += &format!("{} days ", days);
    }
    let hours = duration.num_hours();
    if hours > 0 {
        output += &format!("{} hours ", hours - days * 24);
    }
    let minutes = duration.num_minutes();
    if minutes > 0 && days <= 0 {
        output += &format!("{} minutes ", minutes - hours * 60);
    }
    let seconds = duration.num_seconds();
    if seconds > 0 && hours <= 0 {
        output += &format!("{} seconds", seconds - minutes * 60);
    }

    output
}

pub fn find_command<'a>(
    commands: &HashMap<String, Command>,
    message: &'a str,
) -> Option<(CommandData, Option<Vec<&'a str>>)> {
    let tokens = message.split_whitespace().collect::<Vec<&str>>();
    let mut next_commands = commands;
    for i in 0..tokens.len() {
        if let Some(command) = next_commands.get(tokens[i]) {
            let commands = command.commands.as_ref();
            let data = command.data.as_ref();

            let next = if i + 1 < tokens.len() {
                Some(tokens[i + 1])
            } else {
                None
            };

            if next.is_some() && commands.is_some() && commands.unwrap().contains_key(next.unwrap())
            {
                next_commands = commands.unwrap();
                continue;
            }

            if data.is_some() {
                let mut args: Option<Vec<&str>> = None;
                if tokens.len() - i > 0 {
                    let (_, right) = tokens.split_at(i + 1);
                    if right.len() > 0 {
                        args = Some(right.to_vec())
                    }
                }
                return Some((data.cloned().unwrap(), args));
            } else {
                return None;
            }
        }
    }

    None
}
