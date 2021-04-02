//! Message parsing/writing module
//!
//! * [`irc`](./irc) - parsing raw IRC messages, with Twitch-specific extensions
//!   (not RFC2812 compliant)
//! * [`tmi`](./twitch) - parsing Twitch-specific commands (PRIVMSG, ROOMSTATE,
//!   USERNOTICE, etc.)
//! * [`conn`](./conn) - TMI connection utility
#![feature(str_split_once)]

pub mod conn;
pub mod irc;
pub mod tmi;
pub(crate) mod util;

pub use conn::connect;
pub use conn::Config;
pub use conn::Connection;
pub use tmi::Message;
