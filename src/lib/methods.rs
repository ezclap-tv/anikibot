//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use crate::stream_elements::StreamElementsAPI;
use reqwest::{Error, Response};
use serde_json::Value;

/// Implements the `channels` API methods.
pub struct Channels<'a> {
    api: &'a mut StreamElementsAPI,
}

impl<'a> Channels<'a> {
    /// Creates a new `Channels` object.
    pub fn new(api: &'a mut StreamElementsAPI) -> Self {
        Self { api }
    }
}

// TODO: proper output types
impl<'a> Channels<'a> {
    /// Retrieves the channel information of the API user.
    #[inline(always)]
    pub async fn me(&self) -> Result<Response, Error> {
        self.channel("me").await
    }

    /// Retrieves the channel id of the API user.
    #[inline(always)]
    pub async fn my_id(&self) -> Result<String, Error> {
        self.channel_id("me").await
    }

    /// Retrieves the channel information of the user with the given name.
    pub async fn channel(&self, name_or_id: &str) -> Result<Response, Error> {
        self.api
            .get(&format!("channels/{}/", name_or_id))
            .send()
            .await
    }

    /// Retrieves the channel id of the user with the given name.
    pub async fn channel_id(&self, channel_id: &str) -> Result<String, Error> {
        self.channel(channel_id)
            .await?
            .json::<Value>()
            .await
            .map(|v| v["_id"].as_str().unwrap().to_owned())
    }
}
