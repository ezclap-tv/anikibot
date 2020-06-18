mod util;

use std::sync::atomic::{AtomicBool, Ordering};

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes custom globals
///
/// Panics if called more than once
pub fn init_globals<'a>(lua: &'a mlua::Lua) {
    unsafe {
        if INITIALIZED.load(Ordering::Acquire) {
            panic!("Globals initialized more than once");
        }
        INITIALIZED.store(true, Ordering::Release);
    }

    util::init(lua);
}
