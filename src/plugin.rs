use std::path::{Path, PathBuf};

use mlua::prelude::*;
use walkdir::WalkDir;

use crate::{
    cache, install::Installer, lazy::Handler as LazyHandler, nvim::Nvim,
    runtimepath::RuntimePath, utils::expand_value,
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
            let mut nvim = Nvim::new(lua)?;

            let mut global_rtp: RuntimePath = nvim.get_opt("runtimepath")?;
            global_rtp += &get_rtp(&path);
            nvim.set_opt("runtimepath", &global_rtp)?;

            let mut files = get_plugin_files(&path);
            files.extend(get_ftdetect_files(&path));
            load_files(&mut nvim, &files, None)?;

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

pub fn load_files(
    nvim: &mut Nvim,
    files: &[cache::File],
    filter: Option<&LuaTable>,
) -> LuaResult<()> {
    let mut load_script = String::new();

    for file in files.iter() {
        if let (Some(plugins_filter), Some(stem)) = (filter, file.stem.as_ref()) {
            let stem = stem.as_str();
            if let Ok(LuaValue::Boolean(false)) = plugins_filter.get::<_, LuaValue>(stem)
            {
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

/// `{dir}/plugin/**/*.{vim,lua}`
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
