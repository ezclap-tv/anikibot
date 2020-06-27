use mlua::{Lua, UserData, UserDataMethods, Variadic};
use std::time::Duration;

/// Initializes utility globals
pub fn init_util_globals(lua: &Lua) {
    if let Err(e) = lua.globals().set("util", Util {}) {
        log::error!("Failed to set global object \"util\": {}", e);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Util {}
impl UserData for Util {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_args", |lua, _, va: Variadic<String>| {
            let table = lua.create_table()?;

            table.set("length", va.len() - 2)?;
            table.set("channel", va[0].clone())?;
            table.set("user", va[1].clone())?;

            for i in 0..(va.len() - 2) {
                table.set(i, va[i + 2].clone())?;
            }

            Ok(table)
        });
        methods.add_method("len", |_, _, table: mlua::Table| Ok(table.len()));
        methods.add_method("info", |_, _, va: Variadic<mlua::Value<'lua>>| {
            log::info!(
                "[ LUA ] {}",
                va.into_iter()
                    .map(|v| lua_value_to_string(&v, true))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            Ok(())
        });
        methods.add_method("error", |_, _, va: Variadic<mlua::Value<'lua>>| {
            log::error!(
                "[ LUA ] {}",
                va.into_iter()
                    .map(|v| lua_value_to_string(&v, true))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            Ok(())
        });
        methods.add_method("debug", |_, _, va: Variadic<mlua::Value<'lua>>| {
            log::debug!(
                "[ LUA ] {}",
                va.into_iter()
                    .map(|v| lua_value_to_string(&v, true))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            Ok(())
        });
        methods.add_method("dbg", |_, _, va: Variadic<mlua::Value<'lua>>| {
            log::debug!(
                "[ LUA ] {}",
                va.iter()
                    .map(|v| lua_value_to_string(&v, true))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            Ok(va)
        });
        methods.add_async_method("wait", |_, _, time: u16| async move {
            log::debug!("[ LUA ] Waiting for {}ms", time);
            tokio::time::delay_for(Duration::from_millis(time as u64)).await;
            Ok(())
        });
    }
}

fn lua_value_to_string<'lua>(v: &mlua::Value<'lua>, is_top_level: bool) -> String {
    match v {
        mlua::Value::Nil => "nil".to_owned(),
        mlua::Value::Boolean(b) => b.to_string(),
        mlua::Value::Integer(i) => i.to_string(),
        mlua::Value::Number(n) => n.to_string(),
        mlua::Value::String(s) => match s.to_str() {
            Ok(s) => {
                if is_top_level {
                    s.to_owned()
                } else {
                    format!("{:?}", s)
                }
            }
            Err(e) => format!("{:?}", e),
        },
        mlua::Value::Table(t) => format!(
            "{{ {} }}",
            t.clone()
                .pairs::<mlua::Value, mlua::Value>()
                .map(|r| match r {
                    Ok((k, v)) => format!(
                        "{}: {}",
                        lua_value_to_string(&k, false),
                        lua_value_to_string(&v, false)
                    ),
                    Err(e) => format!("{:?}", e),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        rest => format!("{:#?}", rest),
    }
}
