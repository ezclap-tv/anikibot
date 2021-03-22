use std::collections::HashSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub channels: HashSet<String>,
    pub management: HashSet<String>,
}

impl BotConfig {
    pub fn get() -> BotConfig {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("bot")).unwrap();

        match config.try_into::<BotConfig>() {
            Err(err) => panic!(err.to_string()),
            Ok(config) => config,
        }
    }
}
