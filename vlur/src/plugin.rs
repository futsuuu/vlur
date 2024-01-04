use std::path::{Path, PathBuf};

use log::error;
use mlua::prelude::*;
use walkdir::WalkDir;

use crate::{
    cache, install::Installer, lazy::Handler as LazyHandler, runtimepath::RuntimePath,
    utils::expand_value,
};

pub struct Plugin<'lua> {
    path: PathBuf,
    lazy: Option<LuaTable<'lua>>,
    install: Option<Installer<'lua>>,
}

impl<'lua> FromLua<'lua> for Plugin<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;

        expand_value!(table, {
            path: String,
            lazy: Option<LuaTable>,
            install: Option<Installer>,
        });
        let r = Self {
            path: PathBuf::from(path),
            lazy,
            install,
        };

        Ok(r)
    }
}

impl<'lua> Plugin<'lua> {
    pub fn add_to_rtp(&self, runtimepath: &mut RuntimePath, cache: &mut cache::Cache) {
        if cache.is_valid {
            if let Some(rtp) = cache.inner.runtimepaths.get(self.path.to_str().unwrap())
            {
                *runtimepath += &rtp;
                return;
            }
        }

        let rtp = get_rtp(&self.path);
        *runtimepath += &rtp;

        cache.is_valid = false;
        cache
            .inner
            .runtimepaths
            .insert(self.path.to_str().unwrap().to_string(), rtp);
    }

    #[inline]
    pub fn get_lazy_handlers(
        &self,
    ) -> Option<LuaTableSequence<'lua, LazyHandler<'lua>>> {
        self.lazy.clone().map(|t| t.sequence_values())
    }

    pub fn setup_installer(&self) -> LuaResult<Option<&Installer<'lua>>> {
        let Some(ref installer) = self.install else {
            return Ok(None);
        };
        if !installer.setup(&self.path)? {
            return Ok(Some(installer));
        }
        Ok(None)
    }

    pub fn get_loader(&self, lua: &'lua Lua) -> LuaResult<LuaFunction<'lua>> {
        let path = self.path.clone();

        let loader = move |lua, _: ()| {
            let mut global_rtp: RuntimePath = vlur_bridge::get_opt(lua, "runtimepath")?;
            global_rtp += &get_rtp(&path);
            vlur_bridge::set_opt(lua, "runtimepath", &global_rtp)?;

            let plugin_files = get_plugin_files(&path);
            let ftdetect_files = get_ftdetect_files(&path);
            plugin_files
                .into_iter()
                .chain(ftdetect_files)
                .for_each(|file| {
                    if file.loader.load(lua).is_err() {
                        error!("failed to load the file");
                    }
                });

            Ok(())
        };

        lua.create_function(loader)
    }
}

fn get_rtp(path: &Path) -> RuntimePath {
    let mut rtp = RuntimePath::default();

    if path.exists() {
        rtp.push(path.to_str().unwrap(), false);

        let after_path = path.join("after");
        if after_path.exists() {
            rtp.push(after_path.to_str().unwrap(), true);
        }
    }

    rtp
}

/// `{dir}/plugin/**/*.{vim,lua}`
pub fn get_plugin_files(dir: &Path) -> Vec<cache::File> {
    let is_default = dir == vlur_bridge::vimruntime();
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
        let loader = cache::FileLoader::from(path);
        let name = if is_default {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        r.push(cache::File { loader, name });
    }

    r
}

/// `{dir}/ftdetect/*.{vim,lua}`
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
        let loader = cache::FileLoader::from(path);

        r.push(cache::File { loader, name: None });
    }

    r
}

fn is_vim_or_lua(path: &Path) -> bool {
    let Some(path) = path.to_str() else {
        return false;
    };
    path.ends_with(".lua") || path.ends_with(".vim")
}
