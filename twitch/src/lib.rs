//! Message parsing/writing module
//!
//! * [`irc`](./irc) - parsing raw IRC messages, with Twitch-specific extensions
//!   (not RFC2812 compliant)
//! * [`tmi`](./twitch) - parsing Twitch-specific commands (PRIVMSG, ROOMSTATE,
//!   USERNOTICE etc.)
//! * [`client`](./client) - high-level Twitch chat client which encapsulates
//!   the complexity of maintaining a connection to Twitch, and
//!   receiving/sending messages

#![feature(str_split_once)]

/* pub mod conn; */
pub mod irc;
pub mod tmi;
pub(crate) mod util;
