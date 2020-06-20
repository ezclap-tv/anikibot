use super::communication::{APIRequestKind, APIResponse, RequestSender};
use crate::lua::JsonValue;
use mlua::{Lua, ToLua, UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub struct ConsumerYouTubePlaylistAPI {
    tx: RequestSender,
}

impl ConsumerYouTubePlaylistAPI {
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    pub async fn set_playlist<S: Into<String>>(&self, playlist_id: S) -> APIResponse {
        yt_api_send!(
            self,
            APIRequestKind::Playlist_Set {
                id: playlist_id.into()
            }
        )
    }

    pub async fn set_page_size(&self, page_size: usize) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_SetPageSize(page_size))
    }

    pub async fn configure<S: Into<String>>(
        &self,
        playlist_id: S,
        page_size: usize,
    ) -> APIResponse {
        yt_api_send!(
            self,
            APIRequestKind::Playlist_Configure {
                id: playlist_id.into(),
                page_size
            }
        )
    }

    pub async fn get_playlist(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_Get)
    }

    pub async fn get_page_size(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetPageSize)
    }

    pub async fn get_config(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetConfig)
    }

    pub async fn get_playlist_videos(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetPlaylistVideos)
    }
}

fn handle_api_response(
    lua: &Lua,
    response: APIResponse,
) -> mlua::Result<(mlua::Value, mlua::Value)> {
    match response {
        Ok(response) => match response {
            super::communication::APIResponseMessage::Done => {
                Ok((lua_str!(lua, "Done"), mlua::Value::Nil))
            }
            super::communication::APIResponseMessage::Number(n) => {
                Ok((lua_str!(lua, &n.to_string()), mlua::Value::Nil))
            }
            super::communication::APIResponseMessage::Str(s) => {
                Ok((lua_str!(lua, &s), mlua::Value::Nil))
            }
            super::communication::APIResponseMessage::Json(o) => {
                Ok((JsonValue(o).to_lua(lua)?, mlua::Value::Nil))
            }
            super::communication::APIResponseMessage::Config(c) => {
                let table = lua.create_table()?;
                table.set("items_per_page", c.items_per_page)?;
                table.set("next_page", c.next_page)?;
                table.set("number_of_videos", c.number_of_videos)?;
                table.set("playlist_id", c.playlist_id)?;
                Ok((mlua::Value::Table(table), mlua::Value::Nil))
            }
            super::communication::APIResponseMessage::Videos(v) => {
                let table = lua.create_sequence_from(v.into_iter().map(|v| v.into_url()))?;
                Ok((mlua::Value::Table(table), mlua::Value::Nil))
            }
        },
        Err(error) => Ok((
            mlua::Nil,
            mlua::Value::String(lua.create_string(&format!("{}", error))?),
        )),
    }
}

impl UserData for ConsumerYouTubePlaylistAPI {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method(
            "configure",
            |lua, instance, (playlist_id, page_size): (String, usize)| async move {
                handle_api_response(lua, instance.configure(playlist_id, page_size).await)
            },
        );
        methods.add_async_method("get_config", |lua, instance, ()| async move {
            handle_api_response(lua, instance.get_config().await)
        });
        methods.add_async_method("get_playlist_videos", |lua, instance, ()| async move {
            handle_api_response(lua, instance.get_playlist_videos().await)
        });
    }
}
