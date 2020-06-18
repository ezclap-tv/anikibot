use mlua::{Lua, UserData, UserDataMethods, Variadic};

pub fn init<'lua>(lua: &'lua Lua) {
    if let Err(e) = lua.globals().set("util", Util {}) {
        log::error!("Failed to set global object \"util\": {}", e);
    }
}

pub struct Util {}
impl UserData for Util {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_args", |lua, _, va: Variadic<String>| {
            let table = lua.create_table()?;

            table.set("length", va.len())?;
            for i in 0..va.len() {
                table.set(i, va[i].clone())?;
            }

            Ok(table)
        });
    }
}
