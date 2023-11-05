mod runtimepath;

use std::path::Path;

use mlua::{lua_module, Function, Lua, Table};
use walkdir::WalkDir;

pub use runtimepath::RuntimePath;

/// Separator character used for Neovim options.
pub const OPT_SEP: char = ',';

/// Lua module entrypoint.
#[lua_module]
fn vlur(lua: &Lua) -> mlua::Result<Table> {
    let exports = lua.create_table()?;

    #[cfg(debug_assertions)]
    exports.set("debug", true)?;

    exports.set("setup", lua.create_function(setup)?)?;

    Ok(exports)
}

macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    )
}

fn setup(_lua: &Lua, nvim: Table) -> mlua::Result<()> {
    expand_value!(nvim, {
        set_opt: Function,
        get_opt: Function,
        exec: Function,
    });

    // :set noloadplugins
    set_opt.call::<_, ()>(("loadplugins", false))?;

    let mut runtimepath: RuntimePath = get_opt.call("runtimepath")?;
    let packpath: String = get_opt.call("packpath")?;

    for dir in packpath.as_str().split(OPT_SEP) {
        runtimepath.add_package(dir);
    }

    // Current `&runtimepath`:
    //
    // 1. plugins in `&runtimepath`
    // 2. plugins in start packages
    // 3. after plugins in `&runtimepath`
    // 4. after plugins in start packages

    let rtp = runtimepath.to_string();
    set_opt.call::<_, ()>(("runtimepath", runtimepath))?;

    let mut vim_cmd = String::new();

    // Source all plugin files
    //
    // - `{dir}/plugin/**/*.vim`
    // - `{dir}/plugin/**/*.lua`
    for dir in rtp.split(OPT_SEP) {
        let dir = Path::new(dir).join("plugin");
        if !dir.exists() {
            continue;
        }

        let entries = WalkDir::new(dir)
            .min_depth(1)
            .into_iter()
            .filter_entry(|entry| {
                let Some(fname) = entry.file_name().to_str() else {
                    return false;
                };
                if entry.file_type().is_file() {
                    fname.ends_with(".lua") || fname.ends_with(".vim")
                } else {
                    true
                }
            });

        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let Some(path) = entry.path().to_str() else {
                continue;
            };

            vim_cmd += "\nsource ";
            vim_cmd += path;
        }
    }

    exec.call::<_, ()>(vim_cmd)?;

    Ok(())
}
