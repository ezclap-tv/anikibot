use twitch::conn::{Config, Twitch};
use twitch::tmi::Message;

#[tokio::main]
async fn main() {
    let (mut recv, mut sender) = Twitch::connect(Config::default()).await.unwrap();
    sender.send("JOIN #moscowwbish\r\n").await.unwrap();

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("CTRL-C");
                break;
            },
            result = recv.next() => match result {
                Ok(message) => {
                    println!("> {}", message);
                    match Message::parse(message).unwrap() {
                        Message::Ping(_) => sender.send("PONG\r\n").await.unwrap(),
                        Message::Privmsg(message) => {
                            println!("#{} {}: {}", message.channel(), message.user.name, message.text());
                            if message.text().starts_with("!stop") {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}
