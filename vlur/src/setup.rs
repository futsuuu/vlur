use std::path::Path;

use log::trace;
use mlua::prelude::*;

use crate::{
    cache::Cache,
    install::install,
    nvim::{self, Nvim},
    plugin::{get_plugin_files, load_files, Plugin},
    runtimepath::RuntimePath,
};

pub fn setup(lua: &Lua, (plugins, config): (LuaTable, LuaTable)) -> LuaResult<()> {
    trace!("start");

    let mut nvim = Nvim::new(lua)?;
    let cache_file = nvim.cache_dir()?.join("cache");

    // :set noloadplugins
    nvim.set_opt("loadplugins", false)?;

    // `cache.is_valid` is [`true`] if successful to read the cache, otherwise [`false`].
    let mut cache = Cache::read(&cache_file).unwrap_or_default();

    let mut global_rtp: RuntimePath = nvim.get_opt("runtimepath")?;

    trace!("read plugins");
    let mut installers = Vec::new();
    let plugins = plugins
        .pairs::<LuaString, Plugin>()
        .filter_map(|pair| pair.ok())
        .fold(Vec::new(), |mut plugins, (id, plugin)| {
            if let Some(installer) = plugin.setup_installer().unwrap_or_default() {
                installers.push(installer.clone());
            }
            plugins.push((id, plugin));
            plugins
        });

    trace!("install plugins");
    install(installers, 5)?;

    trace!("load plugins");
    for (id, plugin) in plugins {
        let Some(lazy_handlers) = plugin.get_lazy_handlers() else {
            plugin.add_to_rtp(&mut global_rtp, &mut cache);
            continue;
        };

        let loader = plugin.get_loader(lua)?;

        for handler in lazy_handlers {
            let mut handler = handler?;
            handler.start(lua, id.clone(), loader.clone())?;
        }
    }

    trace!("load the &packpath");
    global_rtp += get_rtp_in_packpath(&mut nvim, &mut cache)?;

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

    let vimruntime = nvim::vimruntime();
    let plugins_filter = config.get::<_, LuaTable>("default_plugins").ok();

    trace!("load the &runtimepath");
    for dir in &global_rtp {
        let path = Path::new(dir);
        let plugins_filter = if path.starts_with(&vimruntime) {
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

    trace!("update the cache");
    cache.update(&cache_file).ok();

    trace!("finish setup");

    Ok(())
}

fn get_rtp_in_packpath<'a>(
    nvim: &mut Nvim,
    cache: &'a mut Cache,
) -> LuaResult<&'a RuntimePath> {
    let packpath: String = nvim.get_opt("packpath")?;

    if !cache.is_valid || cache.inner.package.packpath != packpath {
        let mut rtp = RuntimePath::default();
        for dir in packpath.as_str().split(nvim::OPT_SEP) {
            rtp.push_package(dir);
        }
        cache.is_valid = false;
        cache.inner.package.packpath = packpath;
        cache.inner.package.runtimepath = rtp;
    }

    Ok(&cache.inner.package.runtimepath)
}
