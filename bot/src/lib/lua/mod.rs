mod util;

use crate::bot::{init_api_globals, Bot};
use mlua::{Lua, ToLua};
use std::sync::atomic::{AtomicBool, Ordering};
use util::init_util_globals;

// TODO: move this where it makes sense
pub struct JsonValue(pub serde_json::Value);
impl<'lua> ToLua<'lua> for JsonValue {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        match self.0 {
            serde_json::Value::Array(a) => Ok(mlua::Value::Table(
                lua.create_sequence_from(a.into_iter().map(JsonValue))?,
            )),
            serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(b)),
            serde_json::Value::Number(n) => {
                Ok(mlua::Value::Number(n.as_f64().expect("good one dude LULW")))
            }
            serde_json::Value::Object(o) => Ok(mlua::Value::Table(
                lua.create_table_from(o.into_iter().map(|(k, v)| (k, JsonValue(v))))?,
            )),
            serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
            serde_json::Value::Null => Ok(mlua::Value::Nil),
        }
    }
}

#[macro_export]
macro_rules! lua_str {
    ($lua:ident, $str:expr) => {
        mlua::Value::String($lua.create_string($str)?)
    };
}

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);
/// Initializes custom globals
///
/// Panics if called more than once
pub fn init_globals<'a>(lua: &'a mlua::Lua, bot: &'a Bot<'a>) {
    unsafe {
        if INITIALIZED.load(Ordering::Acquire) {
            panic!("Globals initialized more than once");
        }
        INITIALIZED.store(true, Ordering::Release);
    }

    init_globals_for_lua(lua, bot);
}

pub(crate) fn init_globals_for_lua<'a>(lua: &'a mlua::Lua, bot: &'a Bot<'a>) {
    init_util_globals(lua);
    init_api_globals(lua, bot.get_api_storage(), bot.get_bot_info());
}
