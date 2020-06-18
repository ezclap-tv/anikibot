//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use crate::stream_elements::communication::{APIRequestKind, APIResponse, RequestSender};

/// Implements the `channels` API methods.
pub struct Channels {
    tx: RequestSender,
}

impl Channels {
    /// Creates a new `Channels` object.
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    /// Retrieves the channel information of the API user.
    pub async fn me(&self) -> APIResponse {
        api_send!(self, APIRequestKind::Channel_Me)
    }

    /// Retrieves the channel id of the API user.
    pub async fn my_id(&self) -> APIResponse {
        api_send!(self, APIRequestKind::Channel_MyId)
    }

    /// Retrieves the channel information of the user with the given name.
    pub async fn channel<S: Into<String>>(&self, name: S) -> APIResponse {
        api_send!(self, APIRequestKind::Channel_Chan { name: name.into() })
    }

    /// Retrieves the channel id of the user with the given name.
    pub async fn channel_id(&self, name: &str) -> APIResponse {
        api_send!(self, APIRequestKind::Channel_Id { name: name.into() })
    }
}
