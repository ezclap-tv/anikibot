use twitch::{Config, Message};

fn init_logger() -> std::result::Result<(), log::SetLoggerError> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::try_init()
}

#[tokio::main]
async fn main() {
    init_logger().unwrap();

    let mut conn = twitch::connect(Config::default()).await.unwrap();
    conn.sender.join("moscowwbish").await.unwrap();

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::info!("CTRL-C");
                break;
            },
            result = conn.reader.next() => match result {
                Ok(message) => match message {
                    Message::Ping(ping) => conn.sender.pong(ping.arg()).await.unwrap(),
                    Message::Privmsg(message) => {
                        log::info!("#{} {} ({}): {}", message.channel(), message.user.name, message.user.id(), message.text());
                        if message.text().starts_with("!stop") {
                            break;
                        }
                    },
                    _ => ()
                },
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}
