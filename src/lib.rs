mod cache;
mod runtimepath;
mod utils;

use std::{env, fs, path::Path};

use mlua::{lua_module, Function, Lua, Table, Value};
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

fn setup(lua: &Lua, args: Table) -> mlua::Result<()> {
    expand_value!(lua.globals(), {
        print: Function,
    });
    expand_value!(args, {
        nvim: Table,
        config: Table,
    });
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
        let vimruntime = env::var_os("VIMRUNTIME").unwrap();
        let vimruntime = Path::new(&vimruntime);
        let default_plugins = config.get::<_, Table>("default_plugins").ok();
        let mut script = String::new();

        for dir in runtimepath.to_string().split(OPT_SEP) {
            let dir = Path::new(dir);
            load_plugin_files(dir, &mut script, if dir.starts_with(&vimruntime) {
                default_plugins.as_ref()
            } else {
                None
            });
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
fn load_plugin_files(dir: &Path, load_script: &mut String, default_plugins: Option<&Table>) {
    let dir = dir.join("plugin");
    if !dir.exists() {
        return;
    }

    let entries = WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                return true;
            }
            let Some(fname) = entry.file_name().to_str() else {
                return false;
            };
            let Some(default_plugins) = default_plugins else {
                return fname.ends_with(".lua") || fname.ends_with(".vim");
            };
            let Some((stem, ext)) = fname.rsplit_once('.') else {
                return false;
            };
            if ext != "lua" && ext != "vim" {
                return false;
            }
            if let Ok(Value::Boolean(false)) = default_plugins.get::<_, Value>(stem) {
                return false;
            }
            true
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
