//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use super::api::{APIResult, StreamElementsAPI};
use reqwest::Response;
use serde_json::Value;

/// Implements the `channels` API methods.
pub struct Channels<'a> {
    api: &'a StreamElementsAPI,
}

// TODO: proper output types
impl<'a> Channels<'a> {
    /// Creates a new `Channels` object.
    pub fn new(api: &'a StreamElementsAPI) -> Self {
        Self { api }
    }

    /// Retrieves the channel information of the API user.
    #[inline(always)]
    pub async fn me(&self) -> APIResult<Response> {
        self.channel("me").await
    }

    /// Retrieves the channel id of the API user.
    #[inline(always)]
    pub async fn my_id(&self) -> APIResult<String> {
        self.channel_id("me").await
    }

    /// Retrieves the channel information of the user with the given name.
    pub async fn channel(&self, name_or_id: &str) -> APIResult<Response> {
        self.api
            .get(&format!("channels/{}/", name_or_id))
            .send()
            .await
    }

    /// Retrieves the channel id of the user with the given name.
    pub async fn channel_id(&self, channel_id: &str) -> APIResult<String> {
        self.channel(channel_id)
            .await?
            .json::<Value>()
            .await
            .map(|v| v["_id"].as_str().unwrap().to_owned())
    }
}
