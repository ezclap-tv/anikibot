use std::fmt::{self, Display, Formatter};

#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    pub twitch_login: Option<String>,
    pub twitch_token: Option<String>,
}
#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub main_channel: String,
    pub main_channel_prefix: String,
    pub worker_memory_limit: usize,
    pub concurrency: usize,
    pub credentials: Option<Credentials>,
}
#[derive(Clone, serde::Deserialize)]
struct PartialConfig {
    main_channel: Option<String>,
    main_channel_prefix: Option<String>,
    worker_memory_limit: Option<usize>,
    concurrency: Option<usize>,
    credentials: Option<Credentials>,
}
impl Config {
    pub fn init(path: &str) -> Config {
        log::debug!("Loading config from file '{}'", path);
        let cfg = match std::fs::read_to_string(path) {
            Ok(v) => v,
            Err(err) => {
                log::warn!("Failed to read config: {}; Falling back to defaults", err);
                String::new()
            }
        };
        let cfg = match toml::from_str::<PartialConfig>(&cfg) {
            Ok(value) => value.into(),
            Err(err) => {
                panic!("Error while reading config: {}", err);
            }
        };
        log::info!("Using config: {}", cfg);
        cfg
    }

    pub fn twitch(&self) -> twitch::Config {
        twitch::Config {
            membership_data: false,
            credentials: match &self.credentials {
                Some(Credentials {
                    twitch_login,
                    twitch_token: twitch_pass,
                }) if twitch_login.is_some() && twitch_pass.is_some() => twitch::conn::Login::Regular {
                    login: twitch_login.as_ref().unwrap().clone(),
                    token: twitch_pass.as_ref().unwrap().clone(),
                },
                _ => twitch::conn::Login::Anonymous,
            },
        }
    }

    pub fn script(&self) -> script::Config {
        script::Config {
            memory_limit: Some(self.worker_memory_limit),
        }
    }
}

impl From<PartialConfig> for Config {
    fn from(cfg: PartialConfig) -> Config {
        Config {
            main_channel: cfg
                .main_channel
                .unwrap_or_else(|| panic!("Config.main_channel MUST be set")),
            main_channel_prefix: cfg.main_channel_prefix.unwrap_or_else(|| "!".into()),
            worker_memory_limit: cfg.worker_memory_limit.unwrap_or(512 * 1024 * 1024),
            concurrency: cfg.concurrency.unwrap_or_else(num_cpus::get),
            credentials: cfg.credentials,
        }
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Config {{")?;
        writeln!(f, "\tmain_channel = '{:?}',", self.main_channel)?;
        writeln!(f, "\tworker_memory_limit = {},", self.worker_memory_limit)?;
        writeln!(f, "\tconcurrency = {},", self.concurrency)?;
        writeln!(f, "\tcredentials = ...,")?;
        write!(f, "}}")
    }
}
