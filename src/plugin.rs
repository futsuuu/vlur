use std::path::PathBuf;

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

        let rtp = self.get_rtp();
        *runtimepath += &rtp;

        cache.is_valid = false;
        cache
            .inner
            .runtimepaths
            .insert(self.path.to_str().unwrap().to_string(), rtp);
    }

    pub fn lazy_load<'lua>(&self, lua: &'lua Lua, nvim: &mut Nvim<'lua>) -> Result<()> {
        let path = self.path.clone();
        let loader_id = self.path.to_str().unwrap().to_string();
        let rtp = self.get_rtp();

        nvim.set_plugin_loader(
            loader_id.as_str(),
            lua.create_function(move |lua, _: ()| {
                let mut nvim = Nvim::new(lua)?;

                let mut global_rtp: RuntimePath = nvim.get_opt("runtimepath")?;
                global_rtp += &rtp;
                nvim.set_opt("runtimepath", &global_rtp)?;

                let path = path.as_path();
                let mut files = get_plugin_files(path);
                files.extend(get_ftdetect_files(path));
                let files = files.as_slice();

                load_files(&mut nvim, files, None)?;
                Ok(())
            })?,
        )?;

        nvim.create_autocmd(
            "InsertEnter",
            "*",
            lua.create_function(move |lua, _: ()| {
                let mut nvim = Nvim::new(lua)?;
                nvim.get_plugin_loader(loader_id.as_str())?.call(())?;
                Ok(())
            })?,
            true,
        )?;

        Ok(())
    }

    fn get_rtp(&self) -> RuntimePath {
        let mut rtp = RuntimePath::default();

        if self.path.exists() {
            rtp.push(self.path.to_str().unwrap(), false);

            let after_path = self.path.join("after");
            if after_path.exists() {
                rtp.push(after_path.to_str().unwrap(), true);
            }
        }

        rtp
    }
}
