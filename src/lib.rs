mod cache;
mod nvim;
mod runtimepath;
mod utils;

use std::{env, fs, path::Path};

use mlua::{lua_module, Lua, Table, Value};
use walkdir::WalkDir;

use cache::Cache;
use nvim::Nvim;
pub use runtimepath::RuntimePath;
use utils::expand_value;

pub const BUILT_TIME: &str = include_str!(concat!(env!("OUT_DIR"), "/built_time"));
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

fn setup(_lua: &Lua, args: Table) -> mlua::Result<()> {
    expand_value!(args, {
        config: Table,
    });
    expand_value!(args, mut {
        nvim: Nvim,
    });
    let cache_dir = nvim.cache_dir()?;
    let cache_file = &cache_dir.join("cache");

    // :set noloadplugins
    nvim.set_opt("loadplugins", false)?;

    // `cache.is_valid` is [`true`] if successful to read the cache, otherwise [`false`].
    let mut cache = Cache::read(cache_file).unwrap_or_default();

    let mut runtimepath: RuntimePath = nvim.get_opt("runtimepath")?;

    // TODO: Load user plugins

    let packpath: String = nvim.get_opt("packpath")?;

    // Load `&packpath`.
    if !cache.is_valid || cache.inner.package.packpath != packpath {
        let mut rtp = RuntimePath::default();
        for dir in packpath.as_str().split(OPT_SEP) {
            rtp.push_package(dir);
        }
        cache.is_valid = false;
        cache.inner.package.packpath = packpath;
        cache.inner.package.runtimepath = rtp;
    }
    runtimepath += &cache.inner.package.runtimepath;

    // Current `&runtimepath`:
    //
    // 1. plugins in `&runtimepath`
    // 2. plugins in start packages
    // 3. after plugins in `&runtimepath`
    // 4. after plugins in start packages

    // Update `&runtimepath`.
    nvim.set_opt("runtimepath", runtimepath.clone())?;

    let vimruntime = env::var_os("VIMRUNTIME").unwrap();
    let vimruntime = Path::new(&vimruntime);
    let plugins_filter = config.get::<_, Table>("default_plugins").ok();
    let mut load_script = String::new();

    // Load the plugin scripts.
    let rtp = runtimepath.to_string();
    for dir in rtp.split(OPT_SEP) {
        let path = Path::new(dir);
        let plugins_filter = if path.starts_with(vimruntime) {
            plugins_filter.as_ref()
        } else {
            None
        };

        let plugin_files = if let (true, Some(files)) =
            (cache.is_valid, cache.inner.plugins.get(dir))
        {
            files
        } else {
            let files = get_plugin_files(path);
            cache.is_valid = false;
            cache.inner.plugins.insert(dir.to_string(), files);
            cache.inner.plugins.get(dir).unwrap()
        };

        for file in plugin_files {
            if let Some(plugins_filter) = plugins_filter {
                if let Ok(Value::Boolean(false)) =
                    plugins_filter.get::<_, Value>(file.stem.as_str())
                {
                    continue;
                };
            }
            match &file.loader {
                cache::FileLoader::Script(command) => load_script += &command,
            }
        }
    }

    nvim.exec(load_script)?;

    if !cache.is_valid {
        cache.inner.built_time = BUILT_TIME.to_string();
        fs::create_dir_all(cache_dir).ok();
        cache.write(cache_file).ok();
    }

    Ok(())
}

/// Add scripts that load all plugin files to `load_script`.
///
/// - `{dir}/plugin/**/*.vim`
/// - `{dir}/plugin/**/*.lua`
fn get_plugin_files(dir: &Path) -> Vec<cache::File> {
    let dir = dir.join("plugin");
    if !dir.exists() {
        return Vec::new();
    }

    let entries = WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                return true;
            }
            if entry.path().to_str().is_none() {
                return false;
            };
            true
        });

    let mut r = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let loader =
            cache::FileLoader::Script(format!("source {}\n", path.to_str().unwrap()));

        r.push(cache::File { stem, loader });
    }

    r
}
