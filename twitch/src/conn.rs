///! High-level Twitch chat client
///!
///! Features:
///! * Maintaining a connection to Twitch, including PING/PONG, RECONNECT and
/// tmi.twitch.tv going down ! * Async, stream-based interface for
/// sending/receiving messages ! * Automatic rate limiting based on known
/// user/room state
use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use thiserror::Error;
use tmi::write;
use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream;
use tokio_stream::wrappers::LinesStream;

use crate::{irc, tmi};

const TMI_URL_HOST: &str = "irc.chat.twitch.tv";
const TMI_TLS_PORT: u16 = 6697;

// TODO: rate limiting

#[derive(Clone, Debug, PartialEq)]
pub enum Login {
    Anonymous,
    Regular { login: String, token: String },
}

impl Default for Login {
    fn default() -> Self { Login::Anonymous }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Config {
    pub membership_data: bool,
    pub reconnect: bool,
    pub reconnect_timeout: i64,
    pub rate_limit: bool,
    pub credentials: Login,
}

#[allow(clippy::clippy::upper_case_acronyms)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection to Twitch IRC server failed")]
    ConnectionFailed,
    #[error("Encountered an I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Encountered an error while parsing: {0}")]
    Parse(#[from] tmi::parse::Error),
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
    #[error("Timed out")]
    Timeout,
    #[error("Stream closed")]
    StreamClosed,
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! err {
    ($Variant:ident, $msg:expr) => {
Err(err!(bare $Variant, $msg))
    };
    (bare $Variant:ident, $msg:expr) => {
        crate::conn::Error::$Variant(anyhow::anyhow!($msg))
    };
}

fn expected_cap_ack(request_membership_data: bool) -> &'static str {
    if request_membership_data {
        "twitch.tv/commands twitch.tv/tags twitch.tv/membership"
    } else {
        "twitch.tv/commands twitch.tv/tags"
    }
}

async fn connect_tls(host: &str, port: u16) -> Result<TlsStream<TcpStream>> {
    use tokio_rustls::{rustls::ClientConfig, webpki::DNSNameRef, TlsConnector};

    let mut config = ClientConfig::new();
    config.root_store = rustls_native_certs::load_native_certs().expect("Failed to load native certs");
    let config = TlsConnector::from(Arc::new(config));
    let dnsname = DNSNameRef::try_from_ascii_str(host).map_err(|err| anyhow::anyhow!(err))?;
    let stream = TcpStream::connect((host, port))
        .await
        .map_err(|err| anyhow::anyhow!(err))?;
    let out = config
        .connect(dnsname, stream)
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(out)
}

pub struct Receiver {
    stream: LinesStream<BufReader<ReadHalf<TlsStream<TcpStream>>>>,
}
impl Receiver {
    pub async fn next(&mut self) -> Result<String> {
        if let Some(message) = self.stream.next().await {
            Ok(message?)
        } else {
            Err(Error::StreamClosed)
        }
    }
}

pub struct Sender {
    stream: WriteHalf<TlsStream<TcpStream>>,
}
impl Sender {
    pub async fn send(&mut self, message: &str) -> Result<()> {
        self.stream.write_all(message.as_bytes()).await?;
        Ok(())
    }
}

pub struct Twitch;
impl Twitch {
    pub async fn connect(config: Config) -> Result<(Receiver, Sender)> {
        // 1. connect
        let connection: TlsStream<TcpStream> =
            tokio::time::timeout(Duration::from_secs(5), connect_tls(TMI_URL_HOST, TMI_TLS_PORT))
                .await
                .or(Err(Error::Timeout))??;
        let (read, mut write) = split(connection);
        let mut read = LinesStream::new(BufReader::new(read).lines());

        // 2. request capabilities
        // < CAP REQ :twitch.tv/commands twitch.tv/tags [twitch.tv/membership]
        let req = write::cap(false);
        print!("< {}", req.trim());
        write.write_all(req.as_bytes()).await?;
        // 3. wait for CAP * ACK :twitch.tv/commands twitch.tv/tags
        if let Some(line) = read.next().await {
            let line = line?;
            println!("> {}", line);
            match tmi::Message::parse(line)? {
                tmi::Message::Capability(capability) => {
                    if capability.which() != expected_cap_ack(config.membership_data) {
                        return err!(Generic, "Did not receive expected capabilities");
                    }
                }
                _ => {
                    return err!(Generic, "Did not receive expected capabilities");
                }
            }
        }
        // 4. authenticate
        match &config.credentials {
            Login::Anonymous => {
                use rand::Rng;
                // don't need PASS here
                let login = write::nick(&format!("justinfan{}", rand::thread_rng().gen_range(10000..99999)));
                // < NICK <login>
                println!("< {}", login.trim());
                write.write_all(login.as_bytes()).await?;
            }
            Login::Regular { login, token } => {
                let pass = write::pass(token);
                let nick = write::nick(login);
                // < PASS oauth:<token>
                println!("< {}", pass.trim());
                write.write_all(pass.as_bytes()).await?;
                // < NICK <login>
                println!("< {}", nick.trim());
                write.write_all(nick.as_bytes()).await?;
            }
        }
        // 5. wait for response with command `001`
        if let Some(line) = read.next().await {
            let line = line?;
            println!("> {}", line.trim());
            match tmi::Message::parse(line)? {
                tmi::Message::Unknown(msg) => {
                    if msg.cmd != irc::Command::Unknown("001".into()) {
                        return err!(Generic, "Failed to authenticate");
                    }
                }
                _ => {
                    return err!(Generic, "Did not receive expected capabilities");
                }
            }
        }

        Ok((Receiver { stream: read }, Sender { stream: write }))
    }
}
