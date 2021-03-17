//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use super::api::{APIResult, StreamElementsAPI};
use crate::{BackendError, BoxedError};
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
    pub async fn my_id(&self) -> Result<String, BackendError> {
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
    pub async fn channel_id(&self, name: &str) -> Result<String, BackendError> {
        self.channel(name)
            .await
            .map_err(|e| BackendError::from(BoxedError::from(e)))?
            .json::<Value>()
            .await
            .map_err(|e| BackendError::from(BoxedError::from(e)))
            .and_then(|v| {
                v.get("_id")
                    .map(|id| id.as_str().unwrap().to_owned())
                    .ok_or_else(|| {
                        BackendError::from(format!("Failed to fetch the channel id for `{}`", name))
                    })
            })
    }
}
