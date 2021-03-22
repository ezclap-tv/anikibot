use std::collections::HashMap;
use std::ops::Deref;

use chrono::{DateTime, Duration, TimeZone, Utc};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Expected tag '{0}'")]
    MissingTag(String),
    #[error("Missing prefix")]
    MissingPrefix,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct Message<'a> {
    // TODO: Params should just be a range
    pub tags: Tags<'a>,
    pub prefix: Prefix<'a>,
    pub cmd: Command<'a>,
    pub channel: Option<&'a str>,
    pub params: Params<'a>,
    pub source: &'a str,
}

impl<'a> Message<'a> {
    /// Parse a raw IRC Message
    ///
    /// Parses some Twitch-specific things, such as
    /// nick-only prefixes being host-only, or
    /// the #<channel id> always being present
    /// before :params
    pub fn parse(source: &'a str) -> Result<Message<'a>> {
        let (tags, remainder) = Tags::parse(source)?;
        let (prefix, remainder) = Prefix::parse(remainder)?;
        let (cmd, remainder) = Command::parse(remainder)?;
        let (channel, remainder) = parse_channel(remainder);
        let params = Params::parse(remainder);

        Ok(Message {
            tags,
            prefix,
            cmd,
            channel,
            params,
            source,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command<'a> {
    Ping,
    Pong,
    /// Join channel
    Join,
    /// Leave channel
    Part,
    /// Twitch Private Message
    Privmsg,
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
    /// Requesting an IRC capability
    Capability,
    /// Unknown command
    Unknown(&'a str),
}

impl<'a> Command<'a> {
    /// Parses a Twitch IRC command
    ///
    /// Returns (command, remainder)
    pub fn parse(data: &'a str) -> Result<(Command<'a>, &'a str)> {
        use Command::*;
        let data = data.trim_start();
        let end = match data.find(' ') {
            Some(v) => v,
            None => data.len(),
        };
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
            "CAP" => Capability,
            other => Unknown(other),
        };

        Ok((cmd, &data[end..]))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tags<'a>(HashMap<&'a str, &'a str>);

impl<'a> Deref for Tags<'a> {
    type Target = HashMap<&'a str, &'a str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DurationKind {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

impl<'a> Tags<'a> {
    /// Parses IRC tags in the form
    ///
    /// `@key0=[value0];key1=[value1];...;keyN-1=[valueN-1];keyN=[valueN] `
    ///
    /// `[value]`s are optional
    ///
    /// Returns (tags, remainder)
    pub fn parse(data: &'a str) -> Result<(Tags<'a>, &'a str)> {
        let data = match data.strip_prefix('@') {
            Some(v) => v,
            None => data,
        };
        let mut map: HashMap<&'a str, &'a str> = HashMap::new();
        let mut end = 0;

        let mut current_key: Option<&'a str> = None;
        let mut remainder = data;
        let mut local_i = 0;
        let mut previous_char = '*';

        for (i, c) in data.char_indices() {
            match current_key {
                None => match c {
                    // when we parse ';', save key, and parse value
                    '=' => {
                        current_key = Some(&remainder[..local_i]);
                        // remainder is set without this '='
                        remainder = &remainder[(local_i + 1)..];
                        local_i = 0;
                    }
                    _ => {
                        local_i += 1;
                    }
                },
                Some(key) => match c {
                    // when we parse ';', save value, push it into map
                    // and then parse key
                    ';' => {
                        let value = &remainder[..local_i];
                        if !value.is_empty() {
                            map.insert(key, value);
                        }
                        // remainder is set without this ';'
                        remainder = &remainder[(local_i + 1)..];
                        local_i = 0;
                        current_key = None;
                    }
                    // if we parse a ' :', that's the end of tags
                    ':' if previous_char == ' ' => {
                        let value = &remainder[..(local_i - 1)];
                        if !value.trim().is_empty() {
                            map.insert(key, value);
                        }
                        end = i;
                        break;
                    }
                    _ => {
                        local_i += 1;
                    }
                },
            }
            previous_char = c;
        }

        Ok((Tags(map), &data[end..]))
    }

    /// Iterates the tags to find one with key == `key`.
    pub fn get(&self, key: &str) -> Option<&'a str> {
        for (item_key, item_value) in self.0.iter() {
            if key == *item_key {
                return Some(item_value);
            }
        }

        None
    }

    /// Parses a string, transforming all whitespace "\\s" to actual whitespace.
    pub fn get_ns(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| {
            let mut out = String::with_capacity(v.len());
            let mut parts = v.split("\\s").peekable();
            while let Some(part) = parts.next() {
                out.push_str(part);
                if parts.peek().is_some() {
                    out.push(' ');
                }
            }
            out
        })
    }

    /// Parses a number
    pub fn get_number<N>(&self, key: &str) -> Option<N>
    where
        N: std::str::FromStr,
        <N as std::str::FromStr>::Err: std::fmt::Display,
    {
        match self.0.get(key) {
            Some(v) => match v.parse::<N>() {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            None => None,
        }
    }

    /// Parses a numeric bool (0 or 1)
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(v) => match *v {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            },
            None => None,
        }
    }

    /// Parses a comma-separated list of values
    pub fn get_csv(&self, key: &str) -> Option<Vec<&'a str>> {
        match self.0.get(key) {
            Some(v) => Some(v.split(',').filter(|v| !v.is_empty()).collect()),
            None => None,
        }
    }

    /// Parses a millisecond precision UNIX timestamp as a UTC date/time
    pub fn get_date(&self, key: &str) -> Option<DateTime<Utc>> {
        match self.get_number::<i64>(key) {
            Some(v) => Some(Utc.timestamp_millis(v)),
            None => None,
        }
    }

    pub fn get_duration(&self, key: &str, kind: DurationKind) -> Option<Duration> {
        match self.get_number::<i64>(key) {
            Some(v) => match kind {
                DurationKind::Nanoseconds => Some(Duration::nanoseconds(v)),
                DurationKind::Microseconds => Some(Duration::microseconds(v)),
                DurationKind::Milliseconds => Some(Duration::milliseconds(v)),
                DurationKind::Seconds => Some(Duration::seconds(v)),
                DurationKind::Minutes => Some(Duration::minutes(v)),
                DurationKind::Hours => Some(Duration::hours(v)),
                DurationKind::Days => Some(Duration::days(v)),
                DurationKind::Weeks => Some(Duration::weeks(v)),
            },
            None => None,
        }
    }

    /// Like `.get()`, but returns an `Error` in case the key doesn't exist,
    /// or is invalid in some way
    pub fn require(&self, key: &str) -> Result<&'a str> {
        self.get(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_ns()`, but returns an `Error` in case the key doesn't exist,
    /// or is invalid in some way
    pub fn require_ns(&self, key: &str) -> Result<String> {
        self.get_ns(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_number()`, but returns an `Error` in case the key doesn't
    /// exist, or is invalid in some way
    pub fn require_number<N>(&self, key: &str) -> Result<N>
    where
        N: std::str::FromStr,
        <N as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.get_number(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_bool()`, but returns an `Error` in case the key doesn't
    /// exist, or is invalid in some way
    pub fn require_bool(&self, key: &str) -> Result<bool> {
        self.get_bool(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_csv()`, but returns an `Error` in case the key doesn't exist,
    /// or is invalid in some way
    pub fn require_csv(&self, key: &str) -> Result<Vec<&str>> {
        self.get_csv(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_date()`, but returns an `Error` in case the key doesn't
    /// exist, or is invalid in some way
    pub fn require_date(&self, key: &str) -> Result<DateTime<Utc>> {
        self.get_date(key).ok_or_else(|| Error::MissingTag(key.into()))
    }

    /// Like `.get_duration()`, but returns an `Error` in case the key doesn't
    /// exist, or is invalid in some way
    pub fn require_duration(&self, key: &str, kind: DurationKind) -> Result<Duration> {
        self.get_duration(key, kind)
            .ok_or_else(|| Error::MissingTag(key.into()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prefix<'a> {
    pub nick: Option<&'a str>,
    pub user: Option<&'a str>,
    pub host: &'a str,
}

impl<'a> Prefix<'a> {
    /// Parses an IRC prefix in one of the following forms:
    ///
    /// * `host`
    /// * `nick@host`
    /// * `nick!user@host`
    ///
    /// Returns (prefix, remainder)
    pub fn parse(data: &str) -> Result<(Prefix, &str)> {
        if let Some(start) = data.find(':') {
            let end = match data[start..].find(' ') {
                Some(end) => start + end,
                None => start + data[start..].len(),
            };

            let prefix = &data[start + 1..end];

            // on twitch, nick-only is actually host-only (because they're not fully
            // compliant with RFC2812) so in case we don't find '@', we treat
            // the prefix as just the 'host' part
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
            Err(Error::MissingPrefix)
        }
    }
}

pub fn parse_channel(data: &str) -> (Option<&str>, &str) {
    let data = data.trim_start();
    let (mut start, mut end) = (None, data.len());
    for (i, c) in data.char_indices() {
        match c {
            // No channel, because we found the start of :message
            // TODO: write test that takes into account '#' being present in the message
            ':' if start.is_none() => {
                return (None, data);
            }
            // Either we found `end`
            ' ' if start.is_some() => {
                end = i;
                break;
            }
            // or nothing
            ' ' => {
                return (None, data);
            }
            // We found `start`
            '#' => start = Some(i),
            _ => (),
        }
    }
    let (start, end) = match (start, end) {
        (Some(s), e) => (s, e),
        _ => return (None, data),
    };
    let (channel, remainder) = data[start..].split_at(end);
    (Some(&channel[1..]), remainder)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params<'a>(Vec<&'a str>);

impl<'a> Deref for Params<'a> {
    type Target = Vec<&'a str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Params<'a> {
    /// Parse a params list
    ///
    /// Valid form: `[:]param0 [:]param1 [:]param2 [:]param3"
    pub fn parse(data: &str) -> Params {
        Params(
            data.trim_start()
                .split(' ')
                .map(|p| p.strip_prefix(':').unwrap_or(p))
                .filter(|p| !p.is_empty())
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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
    fn parse_missing_prefix() {
        assert_eq!(Error::MissingPrefix, Prefix::parse("").unwrap_err());
    }

    #[test]
    fn parse_command() {
        assert_eq!(Command::Privmsg, Command::parse("PRIVMSG").unwrap().0)
    }

    // TODO: tests for parsing other message types

    #[test]
    fn parse_join() {
        let src = ":test!test@test.tmi.twitch.tv JOIN #channel";

        assert_eq!(
            Message {
                tags: Tags(HashMap::new()),
                prefix: Prefix {
                    nick: Some("test"),
                    user: Some("test"),
                    host: "test.tmi.twitch.tv"
                },
                cmd: Command::Join,
                channel: Some("channel"),
                params: Params(vec![]),
                source: src
            },
            Message::parse(src).unwrap()
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
                tags: Tags(
                    vec![
                        ("color", "#0000FF"),
                        ("display-name", "JuN1oRRRR"),
                        ("id", "e9d998c3-36f1-430f-89ec-6b887c28af36"),
                        ("mod", "0"),
                        ("room-id", "11148817"),
                        ("subscriber", "0"),
                        ("tmi-sent-ts", "1594545155039"),
                        ("turbo", "0"),
                        ("user-id", "29803735"),
                    ]
                    .into_iter()
                    .collect()
                ),
                prefix: Prefix {
                    nick: Some("jun1orrrr"),
                    user: Some("jun1orrrr"),
                    host: "jun1orrrr.tmi.twitch.tv"
                },
                cmd: Command::Privmsg,
                channel: Some("pajlada"),
                params: Params(vec!["dank", "cam"]),
                source: src
            },
            Message::parse(src).unwrap()
        );
    }

    #[test]
    fn parse_whisper_with_emotes() {
        let src = "\
        @badges=;color=#2E8B57;display-name=pajbot ;emotes=25:7-11;message-id=\
        2034;thread-id=40286300_82008718;turbo=0;user-id=82008718;user-type= \
        :pajbot!pajbot@pajbot.tmi.twitch.tv WHISPER randers :Riftey Kappa\
        ";
        assert_eq!(
            Message {
                tags: Tags(
                    vec![
                        ("message-id", "2034"),
                        ("emotes", "25:7-11"),
                        ("turbo", "0"),
                        ("thread-id", "40286300_82008718"),
                        ("user-id", "82008718"),
                        ("color", "#2E8B57"),
                        ("display-name", "pajbot "),
                    ]
                    .into_iter()
                    .collect(),
                ),
                prefix: Prefix {
                    nick: Some("pajbot"),
                    user: Some("pajbot"),
                    host: "pajbot.tmi.twitch.tv",
                },
                cmd: Command::Whisper,
                channel: None,
                params: Params(vec!["randers", "Riftey", "Kappa"]),
                source: src,
            },
            Message::parse(src).unwrap()
        );
    }

    #[test]
    fn parse_whisper_with_action() {
        let src = "\
        @badges=;color=#2E8B57;display-name=pajbot;emotes=25:7-11;message-id=\
        2034;thread-id=40286300_82008718;turbo=0;user-id=82008718;user-type= \
        :pajbot!pajbot@pajbot.tmi.twitch.tv WHISPER randers :\x01ACTION Riftey Kappa\x01\
        ";
        assert_eq!(
            Message {
                tags: Tags(
                    vec![
                        ("message-id", "2034"),
                        ("emotes", "25:7-11"),
                        ("turbo", "0"),
                        ("thread-id", "40286300_82008718"),
                        ("user-id", "82008718"),
                        ("color", "#2E8B57"),
                        ("display-name", "pajbot"),
                    ]
                    .into_iter()
                    .collect(),
                ),
                prefix: Prefix {
                    nick: Some("pajbot"),
                    user: Some("pajbot"),
                    host: "pajbot.tmi.twitch.tv",
                },
                cmd: Command::Whisper,
                channel: None,
                params: Params(vec!["randers", "\x01ACTION", "Riftey", "Kappa\x01"]),
                source: src,
            },
            Message::parse(src).unwrap()
        );
    }

    #[test]
    fn parse_msg_with_extra_semicolons() {
        let src = "\
        @login=supibot;room-id=;target-msg-id=25fd76d9-4731-4907-978e-a391134ebd67;\
        tmi-sent-ts=-6795364578871 :tmi.twitch.tv CLEARMSG #randers :Pong! Uptime: 6h,\
        15m; Temperature: 54.8°C; Latency to TMI: 183ms; Commands used: 795\
        ";
        assert_eq!(
            Message {
                tags: Tags(
                    vec![
                        ("login", "supibot"),
                        ("target-msg-id", "25fd76d9-4731-4907-978e-a391134ebd67"),
                        ("tmi-sent-ts", "-6795364578871")
                    ]
                    .into_iter()
                    .collect(),
                ),
                prefix: Prefix {
                    nick: None,
                    user: None,
                    host: "tmi.twitch.tv",
                },
                cmd: Command::Clearmsg,
                channel: Some("randers"),
                params: Params(vec![
                    "Pong!",
                    "Uptime:",
                    "6h,15m;",
                    "Temperature:",
                    "54.8°C;",
                    "Latency",
                    "to",
                    "TMI:",
                    "183ms;",
                    "Commands",
                    "used:",
                    "795"
                ]),
                source: src,
            },
            Message::parse(src).unwrap()
        )
    }
}
