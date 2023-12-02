mod cache;
pub mod lazy;
mod nvim;
mod plugin;
mod runtimepath;
pub mod utils;

use std::{env, fs, path::Path};

use mlua::{lua_module, Lua, Table, Value};
use walkdir::WalkDir;

pub use cache::Cache;
pub use nvim::Nvim;
use plugin::Plugin;
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

    exports.set("lazy", lazy::handlers(lua)?)?;
    exports.set("setup", lua.create_function(setup)?)?;

    Ok(exports)
}

fn setup(lua: &Lua, args: Table) -> mlua::Result<()> {
    expand_value!(args, {
        plugins: Table,
        config: Table,
    });
    let mut nvim = Nvim::new(lua)?;
    let cache_dir = nvim.cache_dir()?;
    let cache_file = &cache_dir.join("cache");

    // :set noloadplugins
    nvim.set_opt("loadplugins", false)?;

    // `cache.is_valid` is [`true`] if successful to read the cache, otherwise [`false`].
    let mut cache = Cache::read(cache_file).unwrap_or_default();

    let mut global_rtp: RuntimePath = nvim.get_opt("runtimepath")?;

    for plugin in plugins.sequence_values::<Plugin>() {
        let plugin = plugin?;

        let Some(lazy_handlers) = plugin.get_lazy_handlers() else {
            plugin.add_to_rtp(&mut global_rtp, &mut cache);
            continue;
        };

        let id = plugin.get_id(lua)?;
        let loader = plugin.get_loader(lua)?;

        for handler in lazy_handlers {
            handler?.start(lua, id.clone(), loader.clone())?;
        }
    }

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
    global_rtp += &cache.inner.package.runtimepath;

    // Current `&runtimepath`:
    //
    // 1. plugins in `&runtimepath`
    // 2. plugins specified by user
    // 3. plugins in start packages
    // 4. after plugins in `&runtimepath`
    // 5. after plugins specified by user
    // 6. after plugins in start packages

    // Update `&runtimepath`.
    nvim.set_opt("runtimepath", &global_rtp)?;

    let vimruntime = env::var_os("VIMRUNTIME").unwrap();
    let vimruntime = Path::new(&vimruntime);
    let plugins_filter = config.get::<_, Table>("default_plugins").ok();

    for dir in &global_rtp {
        let path = Path::new(dir);
        let plugins_filter = if path.starts_with(vimruntime) {
            plugins_filter.as_ref()
        } else {
            None
        };

        let files = if let (true, Some(files)) =
            (cache.is_valid, cache.inner.plugins.get(dir))
        {
            files
        } else {
            let files = get_plugin_files(path);
            cache.is_valid = false;
            cache.inner.plugins.insert(dir.to_string(), files);
            cache.inner.plugins.get(dir).unwrap()
        };

        load_files(&mut nvim, files.as_slice(), plugins_filter)?;
    }

    if !cache.is_valid {
        cache.inner.built_time = BUILT_TIME.to_string();
        if cache_file.exists() {
            fs::remove_file(cache_file).ok();
        } else {
            fs::create_dir_all(cache_dir).ok();
            cache.write(cache_file).ok();
        }
    }

    Ok(())
}

pub fn load_files(
    nvim: &mut Nvim,
    files: &[cache::File],
    filter: Option<&Table>,
) -> mlua::Result<()> {
    let mut load_script = String::new();

    for file in files.iter() {
        if let (Some(plugins_filter), Some(stem)) = (filter, file.stem.as_ref()) {
            let stem = stem.as_str();
            if let Ok(Value::Boolean(false)) = plugins_filter.get::<_, Value>(stem) {
                continue;
            };
        }
        match &file.loader {
            cache::FileLoader::Script(command) => load_script += &command,
        }
    }

    nvim.exec(load_script)?;

    Ok(())
}

/// - `{dir}/plugin/**/*.{vim,lua}`
pub fn get_plugin_files(dir: &Path) -> Vec<cache::File> {
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
            is_vim_or_lua(entry.path())
        });

    let mut r = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let loader = cache::FileLoader::Script(format!("source {}\n", path.display()));

        r.push(cache::File {
            loader,
            stem: Some(stem),
        });
    }

    r
}

/// - `{dir}/ftdetect/*.{vim,lua}`
pub fn get_ftdetect_files(dir: &Path) -> Vec<cache::File> {
    let dir = dir.join("ftdetect");
    if !dir.exists() {
        return Vec::new();
    }

    let entries = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                return false;
            }
            is_vim_or_lua(entry.path())
        });

    let mut r = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let loader = cache::FileLoader::Script(format!("source {}\n", path.display()));

        r.push(cache::File { loader, stem: None });
    }

    r
}

fn is_vim_or_lua(path: &Path) -> bool {
    let Some(path) = path.to_str() else {
        return false;
    };
    path.ends_with(".lua") || path.ends_with(".vim")
}
