//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/

use super::handle_api_response;

use crate::stream_elements::communication::{APIRequestKind, APIResponse, RequestSender};
use mlua::{UserData, UserDataMethods};

/// Implements the `channels` API methods.
#[derive(Clone)]
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
    pub async fn channel_id<S: Into<String>>(&self, name: S) -> APIResponse {
        api_send!(self, APIRequestKind::Channel_Id { name: name.into() })
    }
}

impl UserData for Channels {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("me", |lua, instance, _: ()| async move {
            handle_api_response(lua, instance.me().await)
        });
        methods.add_async_method("my_id", |lua, instance, _: ()| async move {
            handle_api_response(lua, instance.my_id().await)
        });
        methods.add_async_method("channel", |lua, instance, name: String| async move {
            handle_api_response(lua, instance.channel(name).await)
        });
        methods.add_async_method("channel_id", |lua, instance, name: String| async move {
            handle_api_response(lua, instance.channel_id(name).await)
        });
    }
}
