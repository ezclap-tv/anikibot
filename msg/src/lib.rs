#![feature(str_split_once)]

use std::{fmt, ops::Deref};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Invalid tag '{0}'")]
    InvalidTag(String),
    #[error("Missing prefix")]
    MissingPrefix,
    #[error("Invalid command '{0}'")]
    InvalidCommand(String),
    #[error("Unknown parse error")]
    Unknown,
}

// TODO: consider treating the '#channel' param separately from other params, it would simplify some code below.

#[derive(Clone, Debug, PartialEq)]
pub struct Message<'a> {
    params: Params<'a>,
    cmd: Command,
    tags: Tags<'a>,
    prefix: Prefix<'a>,
    source: &'a str,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Ping,
    Pong,
    /// Join channel
    Join,
    /// Leave channel
    Part,
    /// Twitch Private Message
    Privmsg,
    /// Change IRC nickname
    Nick,
    /// Submit IRC password
    Pass,
    // Twitch extensions
    /// Send message to a single user
    Whisper,
    /// Purge a user's messages
    Clearchat,
    /// Single message removal
    Clearmsg,
    /// Sent upon successful authentication (PASS/NICK command)
    GlobalUserState,
    /// Channel starts or stops host mode
    HostTarget,
    /// General notices from the server
    Notice,
    /// Rejoins channels after a restart
    Reconnect,
    /// Identifies the channel's chat settings
    RoomState,
    /// Announces Twitch-specific events to the channel
    UserNotice,
    /// Identifies a user's chat settings or properties
    UserState,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Command::Ping => "PING",
                Command::Pong => "PONG",
                Command::Join => "JOIN",
                Command::Part => "PART",
                Command::Privmsg => "PRIVMSG",
                Command::Nick => "NICK",
                Command::Pass => "PASS",
                Command::Whisper => "WHISPER",
                Command::Clearchat => "CLEARCHAT",
                Command::Clearmsg => "CLEARMSG",
                Command::GlobalUserState => "GLOBALUSERSTATE",
                Command::HostTarget => "HOSTTARGET",
                Command::Notice => "NOTICE",
                Command::Reconnect => "RECONNECT",
                Command::RoomState => "ROOMSTATE",
                Command::UserNotice => "USERNOTICE",
                Command::UserState => "USERSTATE",
            }
        )
    }
}

impl Command {
    /// Parses a Twitch IRC command
    ///
    /// Returns (command, remainder)
    pub fn parse(data: &str) -> Result<(Command, &str), ParseError> {
        use Command::*;
        let data = data.trim_start();
        let end = match data.find(' ') {
            Some(v) => v,
            None => data.len(),
        };
        println!("{:#?}", data);
        let cmd = &data[..end];
        let cmd = match cmd {
            "PING" => Ping,
            "PONG" => Pong,
            "JOIN" => Join,
            "PART" => Part,
            "PRIVMSG" => Privmsg,
            "WHISPER" => Whisper,
            "CLEARCHAT" => Clearchat,
            "CLEARMSG" => Clearmsg,
            "GLOBALUSERSTATE" => GlobalUserState,
            "HOSTTARGET" => HostTarget,
            "NOTICE" => Notice,
            "RECONNECT" => Reconnect,
            "ROOMSTATE" => RoomState,
            "USERNOTICE" => UserNotice,
            "USERSTATE" => UserState,
            _ => return Err(ParseError::InvalidCommand(cmd.to_owned())),
        };

        Ok((cmd, &data[end..]))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tags<'a>(Vec<(&'a str, &'a str)>);

impl<'a> Deref for Tags<'a> {
    type Target = Vec<(&'a str, &'a str)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> fmt::Display for Tags<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@")?;
        let mut iter = self.iter().peekable();
        while let Some((key, value)) = iter.next() {
            write!(
                f,
                "{key}={value}{semi}",
                key = key,
                value = value,
                semi = if iter.peek().is_some() { ";" } else { "" }
            )?;
        }

        Ok(())
    }
}

impl<'a> Tags<'a> {
    /// Parses IRC tags in the form
    ///
    /// `@key0=value0;key1=value1;...;keyN=valueN; `
    ///
    /// Returns (tags, remainder)
    pub fn parse(data: &'a str) -> Result<(Tags<'a>, &'a str), ParseError> {
        let mut end = 0;
        let collection = match data.strip_prefix('@') {
            Some(data) => {
                // message includes tags, retrieve them
                let mut collection: Vec<(&'a str, &'a str)> = Vec::new();
                let mut tag_pair_iter = data.split(';').peekable();
                while let Some(mut tag_pair) = tag_pair_iter.next() {
                    if tag_pair_iter.peek().is_none() {
                        // strip the non-tag part from the last part of the message
                        // e.g. "user-type= :jun1orrrr!jun1orrrr@jun1orrrr.tmi.twitch.tv PRIVMSG #pajlada :dank cam"
                        //   -> "user-type= "
                        end = match tag_pair.find(' ') {
                            Some(idx) => idx,
                            None => tag_pair.len(),
                        };

                        tag_pair = &tag_pair[..end];
                    }
                    match tag_pair.split_once('=') {
                        Some((key, value)) => collection.push((key, value)),
                        None => return Err(ParseError::InvalidTag(tag_pair.to_owned())),
                    };
                }

                collection
            }
            None => Vec::new(),
        };

        Ok((Tags(collection), &data[end..]))
    }

    /// Iterates the tags to find one with key == `key`.
    pub fn get(&self, key: &str) -> Option<&'a str> {
        for (item_key, item_value) in self.iter() {
            if key == *item_key {
                return Some(item_value);
            }
        }

        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prefix<'a> {
    nick: Option<&'a str>,
    user: Option<&'a str>,
    host: &'a str,
}

impl<'a> fmt::Display for Prefix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":{nick}{excl}{user}{at}{host}",
            nick = self.nick.unwrap_or(""),
            excl = if self.nick.is_some() && self.user.is_some() {
                "!"
            } else {
                ""
            },
            user = self.user.unwrap_or(""),
            at = if self.nick.is_some() { "@" } else { "" },
            host = self.host
        )
    }
}

impl<'a> Prefix<'a> {
    /// Parses an IRC prefix in one of the following forms:
    ///
    /// * `host`
    /// * `nick@host`
    /// * `nick!user@host`
    ///
    /// Returns (prefix, remainder)
    pub fn parse(data: &str) -> Result<(Prefix, &str), ParseError> {
        if let Some(start) = data.find(':') {
            let end = match data[start..].find(' ') {
                Some(end) => start + end,
                None => start + data[start..].len(),
            };

            let prefix = &data[start + 1..end];

            // on twitch, nick-only is actually host-only (because they're not fully compliant with RFC2812)
            // so in case we don't find '@', we treat the prefix as just the 'host' part
            let (nick, user, host) = match prefix.split_once('@') {
                Some((nick_and_user, host)) => match nick_and_user.split_once('!') {
                    // case: 'nick!user@host'
                    Some((nick, user)) => (Some(nick), Some(user), host),
                    // case: 'nick@host'
                    None => (Some(nick_and_user), None, host),
                },
                // case: 'host'
                None => (None, None, prefix),
            };

            Ok((Prefix { nick, user, host }, &data[end..]))
        } else {
            Err(ParseError::MissingPrefix)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params<'a>(Vec<&'a str>);

impl<'a> Deref for Params<'a> {
    type Target = Vec<&'a str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> fmt::Display for Params<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: this would be simplified if #channel param was handled separate from other params
        let mut previous_param_was_channel = false;
        let mut iter = self.iter().peekable();
        while let Some(param) = iter.next() {
            write!(
                f,
                "{colon}{param}{trailing_space}",
                colon = if previous_param_was_channel {
                    previous_param_was_channel = false;
                    ":"
                } else {
                    ""
                },
                param = param,
                trailing_space = if iter.peek().is_some() { " " } else { "" }
            )?;

            // after writing the #channel param, the next one must be prefixed with :
            if param.starts_with('#') {
                previous_param_was_channel = true;
            }
        }

        Ok(())
    }
}

impl<'a> Params<'a> {
    pub fn parse(data: &str) -> Params {
        Params(
            data.split(' ')
                .map(|p| p.strip_prefix(':').unwrap_or(p))
                .filter(|p| !p.is_empty())
                .collect(),
        )
    }
}

impl<'a> Message<'a> {
    pub fn parse(source: &str) -> Result<Message, ParseError> {
        let (tags, remainder) = Tags::parse(source)?;
        let (prefix, remainder) = Prefix::parse(remainder)?;
        let (cmd, remainder) = Command::parse(remainder)?;
        let params = Params::parse(remainder);

        Ok(Message {
            params,
            cmd,
            tags,
            prefix,
            source,
        })
    }
}

impl<'a> fmt::Display for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.tags, self.prefix, self.cmd, self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prefix_host_only() {
        // :test.tmi.twitch.tv
        assert_eq!(
            Prefix {
                nick: None,
                user: None,
                host: "test.tmi.twitch.tv"
            },
            Prefix::parse(":test.tmi.twitch.tv").unwrap().0
        );
    }

    #[test]
    fn write_prefix_host_only() {
        // :test.tmi.twitch.tv
        assert_eq!(
            ":test.tmi.twitch.tv",
            &format!(
                "{}",
                Prefix {
                    nick: None,
                    user: None,
                    host: "test.tmi.twitch.tv"
                }
            )
        );
    }

    #[test]
    fn parse_prefix_host_and_nick() {
        // :test@test.tmi.twitch.tv
        assert_eq!(
            Prefix {
                nick: Some("test"),
                user: None,
                host: "test.tmi.twitch.tv"
            },
            Prefix::parse(":test@test.tmi.twitch.tv").unwrap().0
        );
    }

    #[test]
    fn write_prefix_host_and_nick() {
        assert_eq!(
            ":test@test.tmi.twitch.tv",
            &format!(
                "{}",
                Prefix {
                    nick: Some("test"),
                    user: None,
                    host: "test.tmi.twitch.tv"
                }
            )
        );
    }

    #[test]
    fn parse_prefix_full() {
        // :test!test@test.tmi.twitch.tv
        assert_eq!(
            Prefix {
                nick: Some("test"),
                user: Some("test"),
                host: "test.tmi.twitch.tv"
            },
            Prefix::parse(":test!test@test.tmi.twitch.tv").unwrap().0
        );
    }

    #[test]
    fn write_prefix_full() {
        assert_eq!(
            ":test!test@test.tmi.twitch.tv",
            format!(
                "{}",
                Prefix {
                    nick: Some("test"),
                    user: Some("test"),
                    host: "test.tmi.twitch.tv"
                }
            )
        )
    }

    #[test]
    fn parse_missing_prefix() {
        assert_eq!(ParseError::MissingPrefix, Prefix::parse("").unwrap_err());
    }

    #[test]
    fn parse_command() {
        assert_eq!(Command::Privmsg, Command::parse("PRIVMSG").unwrap().0)
    }

    #[test]
    fn write_command() {
        assert_eq!("PRIVMSG", format!("{}", Command::Privmsg))
    }

    #[test]
    fn parse_invalid_command() {
        assert_eq!(
            ParseError::InvalidCommand("PRIV_MSG".to_owned()),
            Command::parse("PRIV_MSG").unwrap_err()
        )
    }

    #[test]
    fn parse_full_privmsg() {
        let src = "\
            @badge-info=;\
            badges=;\
            color=#0000FF;\
            display-name=JuN1oRRRR;\
            emotes=;\
            flags=;\
            id=e9d998c3-36f1-430f-89ec-6b887c28af36;\
            mod=0;\
            room-id=11148817;\
            subscriber=0;\
            tmi-sent-ts=1594545155039;\
            turbo=0;\
            user-id=29803735;\
            user-type= \
            :jun1orrrr!jun1orrrr@jun1orrrr.tmi.twitch.tv PRIVMSG #pajlada :dank cam\
        ";
        assert_eq!(
            Message {
                params: Params(vec!["#pajlada", "dank", "cam"]),
                cmd: Command::Privmsg,
                tags: Tags(vec![
                    ("badge-info", ""),
                    ("badges", ""),
                    ("color", "#0000FF"),
                    ("display-name", "JuN1oRRRR"),
                    ("emotes", ""),
                    ("flags", ""),
                    ("id", "e9d998c3-36f1-430f-89ec-6b887c28af36"),
                    ("mod", "0"),
                    ("room-id", "11148817"),
                    ("subscriber", "0"),
                    ("tmi-sent-ts", "1594545155039"),
                    ("turbo", "0"),
                    ("user-id", "29803735"),
                    ("user-type", ""),
                ]),
                prefix: Prefix {
                    nick: Some("jun1orrrr"),
                    user: Some("jun1orrrr"),
                    host: "jun1orrrr.tmi.twitch.tv"
                },
                source: src
            },
            Message::parse(src).unwrap()
        );
    }

    #[test]
    fn write_full_privmsg() {
        assert_eq!(
            "\
            @badge-info=;\
            badges=;\
            color=#0000FF;\
            display-name=JuN1oRRRR;\
            emotes=;\
            flags=;\
            id=e9d998c3-36f1-430f-89ec-6b887c28af36;\
            mod=0;\
            room-id=11148817;\
            subscriber=0;\
            tmi-sent-ts=1594545155039;\
            turbo=0;\
            user-id=29803735;\
            user-type= \
            :jun1orrrr!jun1orrrr@jun1orrrr.tmi.twitch.tv PRIVMSG #pajlada :dank cam\
            ",
            &format!(
                "{}",
                Message {
                    params: Params(vec!["#pajlada", "dank", "cam"]),
                    cmd: Command::Privmsg,
                    tags: Tags(vec![
                        ("badge-info", ""),
                        ("badges", ""),
                        ("color", "#0000FF"),
                        ("display-name", "JuN1oRRRR"),
                        ("emotes", ""),
                        ("flags", ""),
                        ("id", "e9d998c3-36f1-430f-89ec-6b887c28af36"),
                        ("mod", "0"),
                        ("room-id", "11148817"),
                        ("subscriber", "0"),
                        ("tmi-sent-ts", "1594545155039"),
                        ("turbo", "0"),
                        ("user-id", "29803735"),
                        ("user-type", ""),
                    ]),
                    prefix: Prefix {
                        nick: Some("jun1orrrr"),
                        user: Some("jun1orrrr"),
                        host: "jun1orrrr.tmi.twitch.tv"
                    },
                    source: ""
                }
            )
        )
    }
}
