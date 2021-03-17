//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use super::handle_api_response;
use crate::stream_elements::communication::{APIRequestKind, APIResponse, RequestSender};
use mlua::{UserData, UserDataMethods};

/// Implements the `SongRequest` API methods.
#[derive(Clone)]
pub struct SongRequests {
    tx: RequestSender,
}

impl SongRequests {
    /// Creates a new `SongRequests` object.
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    /// Retrieves the song request settings of the API user.
    pub async fn get_settings(&self) -> APIResponse {
        api_send!(self, APIRequestKind::SongReq_Settings)
    }

    /// Retrieves the song request settings for the given `channel_id`.
    pub async fn get_public_settings<S: Into<String>>(&self, channel_id: S) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::SongReq_PublicSettings {
                channel_id: channel_id.into()
            }
        )
    }

    /// Retrieves the currently playing song.
    pub async fn current_song(&self) -> APIResponse {
        api_send!(self, APIRequestKind::SongReq_CurrentSong)
    }

    /// Returns the title of the currently playing song.
    pub async fn current_song_title(&self) -> APIResponse {
        api_send!(self, APIRequestKind::SongReq_CurrentSongTitle)
    }

    /// Queues the given song in the given channel.
    pub async fn queue_song_in_channel<S: Into<String>>(
        &self,
        channel_id: S,
        song_url: S,
    ) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::SongReq_QueueSongInChannel {
                channel_id: channel_id.into(),
                song_url: song_url.into()
            }
        )
    }

    /// Queues the given song in the API user's channel.
    pub async fn queue<S: Into<String>>(&self, song_url: S) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::SongReq_QueueSong {
                song_url: song_url.into()
            }
        )
    }

    /// Queues the given songs in the given channel.
    pub async fn queue_many_in_channel<S: Into<String>>(
        &self,
        channel_id: S,
        song_urls: Vec<String>,
    ) -> APIResponse {
        api_send!(
            self,
            APIRequestKind::SongReq_QueueManyInChannel {
                channel_id: channel_id.into(),
                song_urls
            }
        )
    }

    /// Queues the given songs in the API user's channel.
    pub async fn queue_many(&self, song_urls: Vec<String>) -> APIResponse {
        api_send!(self, APIRequestKind::SongReq_QueueMany { song_urls })
    }
}

impl UserData for SongRequests {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("get_settings", |lua, instance, ()| async move {
            handle_api_response(lua, instance.get_settings().await)
        });
        methods.add_async_method(
            "get_public_settings",
            |lua, instance, channel_id: String| async move {
                handle_api_response(lua, instance.get_public_settings(channel_id).await)
            },
        );
        methods.add_async_method("current_song", |lua, instance, ()| async move {
            handle_api_response(lua, instance.current_song().await)
        });
        methods.add_async_method("current_song_title", |lua, instance, ()| async move {
            handle_api_response(lua, instance.current_song_title().await)
        });
        methods.add_async_method(
            "queue_song_in_channel",
            |lua, instance, (channel_id, song_url): (String, String)| async move {
                handle_api_response(
                    lua,
                    instance.queue_song_in_channel(channel_id, song_url).await,
                )
            },
        );
        methods.add_async_method("queue", |lua, instance, song_url: String| async move {
            handle_api_response(lua, instance.queue(song_url).await)
        });
        methods.add_async_method(
            "queue_many_in_channel",
            |lua, instance, (channel_id, song_urls): (String, Vec<String>)| async move {
                handle_api_response(
                    lua,
                    instance.queue_many_in_channel(channel_id, song_urls).await,
                )
            },
        );
        methods.add_async_method(
            "queue_many",
            |lua, instance, song_urls: Vec<String>| async move {
                handle_api_response(lua, instance.queue_many(song_urls).await)
            },
        );
    }
}
