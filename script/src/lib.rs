#![allow(clippy::new_without_default)]
use std::collections::HashMap;

use lua::{Lua, StdLib};
use mlua as lua;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize Lua: {0}")]
    Init(#[source] lua::Error),
    #[error("Failed to load script: {0}")]
    Load(#[source] lua::Error),
    #[error("Script '{0}' is not loaded")]
    NotLoaded(String),
    #[error("Execution failed: {0}")]
    Exec(#[source] lua::Error),
    #[error("Memory error: {0}")]
    Memory(#[source] lua::Error),
}

fn exec_error(from: lua::Error) -> Error {
    if std::mem::discriminant(&from) == std::mem::discriminant(&mlua::Error::MemoryError(Default::default())) {
        Error::Memory(from)
    } else {
        Error::Exec(from)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Context {
    pub config: Config,
    scripts: HashMap<String, (String, lua::Function<'static>)>,
    ctx: Option<Box<Lua>>,
}

pub type Variadic = lua::Variadic<String>;

/// Used to *temporarily* acquire `&'static mut Lua` to circumvent lifetime
/// bounds
struct LuaStaticGuard<'a> {
    ptr: *mut Lua,
    container: &'a mut Option<Box<Lua>>,
}
impl<'a> LuaStaticGuard<'a> {
    pub unsafe fn new(container: &'a mut Option<Box<Lua>>) -> (LuaStaticGuard<'a>, &'static mut Lua) {
        let leaked = Box::leak(
            container
                .take()
                .expect("Cannot hold two mutable references to Lua at once"),
        );
        let ptr = leaked as *mut _;
        (LuaStaticGuard { ptr, container }, leaked)
    }
}
impl<'a> Drop for LuaStaticGuard<'a> {
    fn drop(&mut self) {
        let ptr = unsafe { Box::from_raw(self.ptr) };
        self.container.replace(ptr);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    pub memory_limit: Option<usize>,
}

impl Context {
    pub fn init(config: Config) -> Result<Context> {
        let libs: StdLib = StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH;
        let lua = Lua::new_with(libs).unwrap();
        if let Some(memory_limit) = config.memory_limit.as_ref() {
            lua.set_memory_limit(*memory_limit).map_err(Error::Init)?;
        }
        Ok(Context {
            config,
            scripts: HashMap::new(),
            ctx: Some(Box::new(lua)),
        })
    }

    pub fn reset(&mut self) {
        // none of the Err cases should arise here
        // clone scripts
        let scripts = self
            .scripts
            .iter()
            .map(|(n, (s, _))| (n.clone(), s.clone()))
            .collect::<Vec<_>>();
        // create new lua state
        let libs: StdLib = StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH;
        let lua = Lua::new_with(libs).unwrap();
        if let Some(memory_limit) = self.config.memory_limit.as_ref() {
            lua.set_memory_limit(*memory_limit)
                .expect("Failed to set memory limit while initializing Lua state");
        }
        std::mem::drop(std::mem::take(&mut self.scripts));
        std::mem::drop(std::mem::replace(&mut self.ctx, Some(Box::new(lua))));
        for (name, code) in scripts {
            self.load(name, &code)
                .expect("Failed to load one of previous commands (?)");
        }
    }

    /// If the script with `name` already exists, it will be replaced.
    pub fn load(&mut self, name: String, script: &str) -> Result<()> {
        // SAFETY: The leaked ptr is returned into the Box when this is dropped.
        // Only one mutable reference to the Lua state exists.
        let (_lg, lua) = unsafe { LuaStaticGuard::new(&mut self.ctx) };
        let chunk = lua.load(script);
        let _ = self
            .scripts
            .insert(name, (script.to_string(), chunk.into_function().map_err(Error::Load)?));
        Ok(())
    }

    pub fn exists(&self, name: &str) -> bool { self.scripts.contains_key(name) }

    /// Unload a script from the context
    ///
    /// If the script with `name` doesn't exist, nothing happens.
    pub fn unload(&mut self, name: &str) { let _ = self.scripts.remove(name); }

    /// Synchronously execute a script. The script must be loaded with
    /// `Engine::load` beforehand. Contents of the script must fully execute
    /// synchronously, too.
    pub fn exec<A: lua::ToLuaMulti<'static>, R: lua::FromLuaMulti<'static>>(
        &mut self,
        name: &str,
        args: A,
    ) -> Result<R> {
        match self.scripts.get(name) {
            Some(script) => Ok(script.1.call(args).map_err(exec_error)?),
            None => Err(Error::NotLoaded(name.into())),
        }
    }

    /// Asynchronously execute a script. The script must be loaded with
    /// `Engine::load` beforehand. Contents of the script may or may not execute
    /// asynchronously.
    pub async fn exec_async<A: lua::ToLuaMulti<'static>, R: lua::FromLuaMulti<'static>>(
        &mut self,
        name: &str,
        args: A,
    ) -> Result<R> {
        match self.scripts.get(name) {
            Some(script) => Ok(script.1.call_async(args).await.map_err(exec_error)?),
            None => Err(Error::NotLoaded(name.into())),
        }
    }

    pub fn eval<A: lua::ToLuaMulti<'static>, R: lua::FromLuaMulti<'static>>(
        &mut self,
        code: &str,
        args: A,
    ) -> Result<R> {
        let chunk = {
            let (_lg, lua) = unsafe { LuaStaticGuard::new(&mut self.ctx) };
            lua.load(&code)
        };
        chunk.call(args).map_err(exec_error)
    }

    pub async fn eval_async<A: lua::ToLuaMulti<'static>, R: lua::FromLuaMulti<'static>>(
        &mut self,
        code: &str,
        args: A,
    ) -> Result<R> {
        let chunk = {
            let (_lg, lua) = unsafe { LuaStaticGuard::new(&mut self.ctx) };
            lua.load(&code)
        };
        Ok(chunk.call_async(args).await.map_err(exec_error)?)
    }

    /// Creates a scope in which you have access to a `&'static mut Lua`.
    /// Use this to register user data, create globals, etc.
    ///
    /// The lifetime of objects created in this scope is the same as the
    /// lifetime of the `Engine`.
    pub fn scope<R: lua::FromLuaMulti<'static>>(
        &mut self,
        body: impl FnOnce(&mut Lua) -> std::result::Result<R, lua::Error>,
    ) -> Result<R> {
        let (_lg, lua) = unsafe { LuaStaticGuard::new(&mut self.ctx) };
        body(lua).map_err(exec_error)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn execution() {
        let mut ctx = Context::init(Config::default()).unwrap();
        ctx.load("test".into(), "return 1+1").unwrap();
        let out: u32 = ctx.exec("test", ()).unwrap();
        assert_eq!(out, 2u32);
    }

    #[tokio::test]
    async fn async_execution() {
        let mut ctx = Context::init(Config::default()).unwrap();
        ctx.load("test".into(), "return 1+1").unwrap();
        let out: u32 = ctx.exec_async("test", ()).await.unwrap();
        assert_eq!(out, 2u32);
    }

    #[test]
    fn eval() {
        let mut ctx = Context::init(Config::default()).unwrap();
        let out: u32 = ctx.eval("return 1+1", ()).unwrap();
        assert_eq!(out, 2u32);
    }

    #[tokio::test]
    async fn async_eval() {
        async fn calc(_lua: &Lua, v: (u32, u32)) -> mlua::Result<u32> {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(v.0 + v.1)
        }

        let mut ctx = Context::init(Config::default()).unwrap();
        ctx.scope(|lua| {
            lua.globals().set("calc", lua.create_async_function(calc)?)?;

            Ok(())
        })
        .unwrap();
        let out: u32 = ctx.eval_async("return calc(...)", (1u32, 1u32)).await.unwrap();
        assert_eq!(out, 2u32);
    }
}
