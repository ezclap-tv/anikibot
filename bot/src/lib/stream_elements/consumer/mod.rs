use mlua::{Lua, ToLua, UserData, UserDataMethods};

use super::communication::{APIResponse, APIResponseMessage, RequestSender};
use crate::lua::JsonValue;
use channels::Channels;
use song_requests::SongRequests;
use stats::Stats;

pub mod channels;
pub mod song_requests;
pub mod stats;

#[derive(Debug, Clone)]
pub struct ConsumerStreamElementsAPI {
    tx: RequestSender,
}

impl ConsumerStreamElementsAPI {
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    #[must_use = "Calling channels() does nothing"]
    pub fn channels(&self) -> Channels {
        Channels::new(self.tx.clone())
    }

    #[must_use = "Calling song_requests() does nothing"]
    pub fn song_requests(&self) -> SongRequests {
        SongRequests::new(self.tx.clone())
    }

    #[must_use = "Calling stats() does nothing"]
    pub fn stats(&self) -> Stats {
        Stats::new(self.tx.clone())
    }
}

impl UserData for ConsumerStreamElementsAPI {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("channels", |_, instance, ()| Ok(instance.channels()));
        methods.add_method("song_requests", |_, instance, ()| {
            Ok(instance.song_requests())
        });
        methods.add_method("stats", |_, instance, ()| Ok(instance.stats()));
    }
}

fn handle_api_response(
    lua: &Lua,
    response: APIResponse,
) -> Result<(mlua::Value, mlua::Value), mlua::Error> {
    match response {
        Ok(response) => match response {
            APIResponseMessage::Json(json) => Ok((JsonValue(json).to_lua(lua)?, mlua::Nil)),
            APIResponseMessage::Str(str) => {
                Ok((mlua::Value::String(lua.create_string(&str)?), mlua::Nil))
            }
        },
        Err(err) => Ok((
            mlua::Nil,
            mlua::Value::String(lua.create_string(&format!("{}", err))?),
        )),
    }
}
