mod cache;
mod runtimepath;
mod utils;

use std::{fs, path::Path};

use mlua::{lua_module, Function, Lua, Table};
use speedy::{Readable as _, Writable as _};
use walkdir::WalkDir;

use cache::Cache;
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

fn setup(_lua: &Lua, nvim: Table) -> mlua::Result<()> {
    expand_value!(nvim, {
        set_opt: Function,
        get_opt: Function,
        exec: Function,
        cache_dir: String,
    });
    let cache_dir = Path::new(&cache_dir);
    let cache_file = &cache_dir.join("cache");

    // :set noloadplugins
    set_opt.call::<_, ()>(("loadplugins", false))?;

    // `cache.is_valid` is [`true`] if successful to read the cache, otherwise [`false`].
    let mut cache = if let Ok(mut cache) = Cache::read_from_file(cache_file) {
        cache.is_valid = true;
        cache
    } else {
        Cache::default()
    };

    let mut runtimepath: RuntimePath = get_opt.call("runtimepath")?;

    // TODO: Load user plugins

    let packpath: String = get_opt.call("packpath")?;

    // Load `&packpath`.
    if !cache.is_valid || cache.packpath != packpath {
        let mut package_rtp = RuntimePath::default();
        for dir in packpath.as_str().split(OPT_SEP) {
            package_rtp.push_package(dir);
        }
        cache.is_valid = false;
        cache.packpath = packpath;
        cache.package_rtp = package_rtp;
    }
    runtimepath += &cache.package_rtp;

    // Current `&runtimepath`:
    //
    // 1. plugins in `&runtimepath`
    // 2. plugins in start packages
    // 3. after plugins in `&runtimepath`
    // 4. after plugins in start packages

    // Update `&runtimepath`.
    set_opt.call::<_, ()>(("runtimepath", runtimepath.clone()))?;

    // Load the plugin scripts.
    if !cache.is_valid || cache.plugin_rtp != runtimepath {
        let mut script = String::new();
        for dir in runtimepath.to_string().split(OPT_SEP) {
            load_plugin_files(dir, &mut script);
        }
        cache.is_valid = false;
        cache.plugin_rtp = runtimepath;
        cache.load_script = script;
    }
    exec.call::<_, ()>(cache.load_script.clone())?;

    if !cache.is_valid {
        fs::create_dir_all(cache_dir).ok();
        cache.write_to_file(cache_file).ok();
    }

    Ok(())
}

/// Add scripts that load all plugin files to `load_script`.
///
/// - `{dir}/plugin/**/*.vim`
/// - `{dir}/plugin/**/*.lua`
fn load_plugin_files(dir: &str, load_script: &mut String) {
    let dir = Path::new(dir).join("plugin");
    if !dir.exists() {
        return;
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

        *load_script += "\nsource ";
        *load_script += path;
    }
}
