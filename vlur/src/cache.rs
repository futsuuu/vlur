use std::{fs, path::Path};

use hashbrown::HashMap;
use log::trace;
use mlua::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};

use crate::runtimepath::RuntimePath;

const CACHE_ID: [u8; 16] = vlur_macros::unique_bytes!();

#[derive(Default)]
pub struct Cache {
    pub is_valid: bool,
    pub inner: Inner,
}

impl Cache {
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        trace!("restore the cache");
        let bytes = fs::read(path)?;
        let inner: Inner = unsafe { rkyv::from_bytes_unchecked(&bytes)? };
        Ok(Self {
            is_valid: inner.id == CACHE_ID,
            inner,
        })
    }

    pub fn update(&mut self, path: &Path) -> anyhow::Result<()> {
        if self.is_valid {
            return Ok(());
        }
        if path.exists() {
            trace!("remove the invalid cache");
            fs::remove_file(path)?;
            return Ok(());
        }
        trace!("create a new cache");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.inner.id = CACHE_ID;
        let bytes = rkyv::to_bytes::<_, 0>(&self.inner)?;
        fs::write(path, bytes)?;

        Ok(())
    }
}

#[derive(Archive, Deserialize, Serialize, Default)]
#[archive()]
pub struct Inner {
    pub id: [u8; 16],

    /// [`RuntimePath`] added by `&packpath`.
    pub package: Package,

    /// The key is the path to the plugin's directory,
    /// and the value is the directory under that directory
    /// to add to `&runtimepath`.
    pub runtimepaths: HashMap<String, RuntimePath>,

    /// All Vim script/Lua files under the `{rtp}/plugin/` directory.
    pub plugins: HashMap<String, Vec<File>>,
}

#[derive(Archive, Deserialize, Serialize, Default)]
#[archive()]
pub struct Package {
    // cache key
    pub packpath: String,
    // cache value
    pub runtimepath: RuntimePath,
}

#[derive(Archive, Deserialize, Serialize, Default)]
#[archive()]
pub struct File {
    pub loader: FileLoader,
    /// Used to disable loading for default plugins.
    pub name: Option<String>,
}

#[derive(Archive, Deserialize, Serialize)]
#[archive()]
pub enum FileLoader {
    // `source <path>`
    Script(String),
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::Script("".into())
    }
}

impl FileLoader {
    pub fn load(&self, lua: &Lua) -> LuaResult<()> {
        match *self {
            FileLoader::Script(ref script) => vlur_bridge::exec(lua, script.as_str())?,
        }
        Ok(())
    }
}

impl<P: AsRef<Path>> From<P> for FileLoader {
    fn from(value: P) -> Self {
        let path = value.as_ref();
        Self::Script(format!("source {}\n", path.display()))
    }
}
