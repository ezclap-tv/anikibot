pub mod settings;
pub mod structs;
pub mod tz;

pub use settings::StatsSettings;

use super::api::{APIResult, StreamElementsAPI};
use structs::*;

pub struct Stats<'a> {
    api: &'a StreamElementsAPI,
    settings: StatsSettings,
}

impl<'a> Stats<'a> {
    pub fn new(api: &'a StreamElementsAPI) -> Self {
        Self {
            api,
            settings: StatsSettings::default(),
        }
    }

    pub fn set_settings(&mut self, settings: StatsSettings) -> &Self {
        self.settings = settings;
        self
    }

    #[inline(always)]
    pub async fn my_stats(&self) -> APIResult<StatsTotals> {
        self.stats_for_channel(&self.api.channel_id()).await
    }

    /// NOTE: Requires the token bearer to have the necessary permissions.
    pub async fn stats_for_channel(&self, channel_id: &str) -> APIResult<StatsTotals> {
        self.api
            .get(&format!(
                "stats/{}?interval={:?}&date={}&tz={}",
                channel_id,
                self.settings.interval,
                self.settings.date.format("%Y-%m-%d"),
                self.settings.timezone as u32
            ))
            .send()
            .await?
            .json::<AllStats>()
            .await
            .map(|stats| stats.totals)
    }
}
