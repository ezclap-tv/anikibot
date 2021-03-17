//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/

use mlua::{UserData, UserDataMethods};

use super::handle_api_response;
use crate::stream_elements::communication::{APIRequestKind, APIResponse, RequestSender};
use crate::stream_elements::stats::StatsSettings;

/// Implements the `Stats` API methods.
#[derive(Clone)]
pub struct Stats {
    tx: RequestSender,
    settings: StatsSettings,
}
impl Stats {
    /// Creates a new `Stats` object.
    pub fn new(tx: RequestSender) -> Self {
        Self {
            tx,
            settings: StatsSettings::default(),
        }
    }

    /// Stores the given settings.
    pub fn with_settings(self, settings: StatsSettings) -> Self {
        Self { settings, ..self }
    }

    /// Retrieves the stats of the API user's channel.
    pub async fn my_stats(&self) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::Stats_MyStats {
                settings: self.settings.clone()
            }
        )
    }

    /// Retrieves the stats of the given channel. Requires the permission to view channel stats.
    pub async fn stats_for_channel<S: Into<String>>(&self, channel_id: S) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::Stats_ChannelStats {
                channel_id: channel_id.into(),
                settings: self.settings.clone()
            }
        )
    }
}

impl UserData for Stats {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("my_stats", |lua, instance, _: ()| async move {
            handle_api_response(lua, instance.my_stats().await)
        });
        methods.add_async_method(
            "stats_for_channel",
            |lua, instance, channel_id: String| async move {
                handle_api_response(lua, instance.stats_for_channel(channel_id).await)
            },
        );
        methods.add_method("settings", |lua, instance, _: ()| {
            let table = lua.create_table()?;
            table.set("interval", instance.settings.interval.to_string())?;
            table.set(
                "date",
                format!("{}", instance.settings.date.format("%Y-%m-%d")),
            )?;
            table.set("tz", instance.settings.timezone as u32)?;
            table.set("tz_name", format!("{:?}", instance.settings.timezone))?;
            Ok(table)
        });
    }
}
