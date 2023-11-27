use std::path::{Path, PathBuf};

use hashbrown::HashSet;
use mlua::{FromLua, Function, Lua, Result, Table, Value};

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

        let loader_executor = lua.create_function(loader_executor)?.bind(loader)?;
        nvim.create_autocmd("InsertEnter", "*", loader_executor, true)?;

        Ok(())
    }
}

fn loader_executor<'lua>(
    lua: &'lua Lua,
    (plugin_loader, ev): (Function, Table),
) -> Result<()> {
    let mut nvim = Nvim::new(lua)?;
    expand_value!(ev, {
        event: mlua::String,
        data: Value,
    });
    let event = event.to_str()?;

    let mut exists_autocmds = Vec::new();
    let mut exists_ids = HashSet::new();
    let mut exists_groups = HashSet::new();
    for autocmd in nvim.get_autocmds(event, "*")? {
        let autocmd = autocmd?;
        if let Some(id) = autocmd.id {
            exists_ids.insert(id);
        }
        if let Some(group) = autocmd.group {
            exists_groups.insert(group);
        }
        exists_autocmds.push(autocmd);
    }

    plugin_loader.call(())?;

    let mut executed_groups = HashSet::new();
    'autocmd: for autocmd in nvim.get_autocmds(event, "*")? {
        let autocmd = autocmd?;
        if let Some(id) = autocmd.id {
            if exists_ids.contains(&id) {
                continue;
            }
        }
        if let Some(group) = autocmd.group {
            if exists_groups.contains(&group) {
                continue;
            }
            if executed_groups.contains(&group) {
                continue;
            }
            executed_groups.insert(group);
        }
        for exists in &exists_autocmds {
            if autocmd == *exists {
                continue 'autocmd;
            }
        }
        nvim.exec_autocmds(event, "*", autocmd.group, data.clone())?;
    }

    Ok(())
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
