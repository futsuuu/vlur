use std::path::Path;

use log::{error, trace};
use mlua::prelude::*;

use crate::{
    cache::Cache,
    install::install,
    plugin::{get_plugin_files, Plugin},
    runtimepath::RuntimePath,
};

pub fn setup(lua: &Lua, (plugins, config): (LuaTable, LuaTable)) -> LuaResult<()> {
    trace!("start");

    let cache_file = vlur_bridge::cache_dir(lua)?.join("cache");

    // :set noloadplugins
    vlur_bridge::set_opt(lua, "loadplugins", false)?;

    // `cache.is_valid` is [`true`] if successful to read the cache, otherwise [`false`].
    let mut cache = Cache::read(&cache_file).unwrap_or_default();

    let mut global_rtp: RuntimePath = vlur_bridge::get_opt(lua, "runtimepath")?;

    trace!("read plugins");
    let mut installers = Vec::new();
    let plugins = plugins
        .pairs::<LuaString, Plugin>()
        .filter_map(|pair| pair.ok())
        .fold(Vec::new(), |mut plugins, (id, plugin)| {
            if let Some(installer) = plugin.setup_installer().unwrap_or_default() {
                installers.push((id.clone(), installer.clone()));
            }
            plugins.push((id, plugin));
            plugins
        });

    if !installers.is_empty() {
        trace!("install plugins");
        install(lua, installers, 5)?;
    }

    trace!("setup plugins");
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

    trace!("add path from &packpath to &runtimepath");
    global_rtp += get_rtp_in_packpath(lua, &mut cache)?;

    // Current `&runtimepath`:
    //
    // 1. plugins in `&runtimepath`
    // 2. plugins specified by user
    // 3. plugins in start packages
    // 4. after plugins in `&runtimepath`
    // 5. after plugins specified by user
    // 6. after plugins in start packages

    // Update `&runtimepath`.
    vlur_bridge::set_opt(lua, "runtimepath", &global_rtp)?;

    let plugins_filter = config.get::<_, LuaTable>("default_plugins").ok();
    let use_filter = plugins_filter.is_some();

    trace!("load plugin files");
    for dir in &global_rtp {
        let path = Path::new(dir);
        let plugins_filter = plugins_filter.as_ref();

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

        files
            .iter()
            .filter(|file| {
                if !use_filter {
                    return true;
                }
                let Some(ref name) = &file.name else {
                    return true;
                };
                let name = name.as_str();
                if let Ok(LuaValue::Boolean(false)) = plugins_filter.unwrap().get(name) {
                    return false;
                }
                true
            })
            .for_each(|file| {
                if file.loader.load(lua).is_err() {
                    error!("failed to load the file");
                }
            });
    }

    cache.update(&cache_file).into_lua_err()?;

    trace!("finish setup");

    Ok(())
}

fn get_rtp_in_packpath<'a>(
    lua: &Lua,
    cache: &'a mut Cache,
) -> LuaResult<&'a RuntimePath> {
    let packpath: String = vlur_bridge::get_opt(lua, "packpath")?;

    if !cache.is_valid || cache.inner.package.packpath != packpath {
        let mut rtp = RuntimePath::default();
        for dir in packpath.as_str().split(vlur_bridge::OPT_SEP) {
            rtp.push_package(dir);
        }
        cache.is_valid = false;
        cache.inner.package.packpath = packpath;
        cache.inner.package.runtimepath = rtp;
    }

    Ok(&cache.inner.package.runtimepath)
}
