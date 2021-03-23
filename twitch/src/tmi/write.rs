// TODO: figure out how to write to String without re-allocating, so String can
// be used as a buffer.

pub fn join(channel: &str) -> String { format!("JOIN #{}\r\n", channel) }

pub fn part(channel: &str) -> String { format!("PART #{}\r\n", channel) }

/// Panics if `message.len() > 510`
pub fn privmsg(channel: &str, message: &str) -> String {
    if message.len() > 510 {
        panic!("Message size limit reached: {}/{}", message.len(), 510);
    }
    format!("PRIVMSG #{} :{}\r\n", channel, message)
}

pub fn cap(with_membership: bool) -> String {
    format!(
        "CAP REQ :twitch.tv/commands twitch.tv/tags{}\r\n",
        if with_membership { " twitch.tv/membership" } else { "" }
    )
}

pub fn pass(token: &str) -> String { format!("PASS oauth:{}\r\n", token) }

pub fn nick(login: &str) -> String { format!("NICK {}\r\n", login) }
