use std::path::{Path, PathBuf};

use mlua::{FromLua, Function, Lua, Result, Table, TableSequence};

use crate::{
    expand_value, get_ftdetect_files, get_plugin_files, lazy, load_files, utils, Cache,
    Nvim, RuntimePath,
};

pub struct Plugin<'lua> {
    path: PathBuf,
    lazy: Option<Table<'lua>>,
}

impl<'lua> FromLua<'lua> for Plugin<'lua> {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        let table = Table::from_lua(value, lua)?;

        expand_value!(table, {
            path: String,
            lazy: Option<Table>,
        });
        let r = Self {
            path: PathBuf::from(path),
            lazy,
        };

        Ok(r)
    }
}

impl<'lua> Plugin<'lua> {
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

    #[inline]
    pub fn get_lazy_handlers(&self) -> Option<TableSequence<'lua, lazy::Handler<'lua>>> {
        self.lazy.clone().map(|t| t.sequence_values())
    }

    #[inline]
    pub fn get_id(&self, lua: &'lua Lua) -> Result<mlua::String<'lua>> {
        let bytes = utils::os_str_as_bytes(&self.path.as_os_str());
        lua.create_string(bytes)
    }

    pub fn get_loader(&self, lua: &'lua Lua) -> Result<Function<'lua>> {
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
