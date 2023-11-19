use std::path::PathBuf;

use mlua::{Result, Table, FromLua, Lua};

use crate::{expand_value, Cache, RuntimePath};

pub struct Plugin {
    path: PathBuf,
}

impl<'lua> FromLua<'lua> for Plugin {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        let table = Table::from_lua(value, lua)?;

        expand_value!(table, {
            path: String,
        });
        let r = Self {
            path: PathBuf::from(path),
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
