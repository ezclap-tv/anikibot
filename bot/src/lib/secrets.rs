use serde::Deserialize;
use twitchchat::UserConfig;

#[derive(Deserialize)]
pub struct Secrets {
    pub name: String,
    pub oauth_token: String,
    pub stream_elements_jwt_token: Option<String>,
    pub youtube_api_key: Option<String>,
}

impl Secrets {
    pub fn get() -> Secrets {
        let mut secrets = config::Config::default();
        secrets.merge(config::File::with_name("secrets")).unwrap();

        match secrets.try_into::<Secrets>() {
            Err(err) => panic!(err.to_string()),
            Ok(secrets) => secrets,
        }
    }
}

impl Into<UserConfig> for Secrets {
    fn into(self) -> UserConfig {
        twitchchat::UserConfig::builder()
            .name(&self.name)
            .token(&self.oauth_token)
            .build()
            .unwrap()
    }
}
