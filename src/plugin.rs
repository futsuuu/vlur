use std::path::{Path, PathBuf};

use mlua::{FromLua, Lua, Result, Table};

use crate::{
    expand_value, get_ftdetect_files, get_plugin_files, load_files, Cache, Nvim,
    RuntimePath,
};

pub struct Plugin {
    path: PathBuf,
    pub lazy: bool,
}

impl<'lua> FromLua<'lua> for Plugin {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        let table = Table::from_lua(value, lua)?;

        expand_value!(table, {
            path: String,
            lazy: Option<bool>,
        });
        let r = Self {
            path: PathBuf::from(path),
            lazy: lazy.unwrap_or_default(),
        };
        Ok(r)
    }
}

impl Plugin {
    pub fn add_to_rtp(&self, runtimepath: &mut RuntimePath, cache: &mut Cache) {
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

    pub fn lazy_load<'lua>(&self, lua: &'lua Lua, nvim: &mut Nvim<'lua>) -> Result<()> {
        let path = self.path.clone();

        let loader = lua.create_function(move |lua, _: ()| {
            let mut nvim = Nvim::new(lua)?;

            let mut global_rtp: RuntimePath = nvim.get_opt("runtimepath")?;
            global_rtp += &get_rtp(&path);
            nvim.set_opt("runtimepath", &global_rtp)?;

            let mut files = get_plugin_files(&path);
            files.extend(get_ftdetect_files(&path));
            load_files(&mut nvim, &files, None)?;

            Ok(())
        })?;

        let loader_id = self.path.to_str().unwrap().to_string();
        nvim.set_plugin_loader(&loader_id, loader)?;

        let loader_executor = lua.create_function(move |lua, _: ()| {
            let mut nvim = Nvim::new(lua)?;
            let loader = nvim.get_plugin_loader(&loader_id)?;
            loader.call(())?;
            Ok(())
        })?;

        nvim.create_autocmd("InsertEnter", "*", loader_executor, true)?;

        Ok(())
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
