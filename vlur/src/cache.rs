use std::{fs, path::Path};

use hashbrown::HashMap;
use rkyv::{Archive, Deserialize, Serialize};

use crate::runtimepath::RuntimePath;

const COMPILED_TIME: [u8; 16] = vlur_macros::unique_bytes!();

#[derive(Default)]
pub struct Cache {
    pub is_valid: bool,
    pub inner: Inner,
}

impl Cache {
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        let bytes = fs::read(path)?;
        let inner: Inner = unsafe { rkyv::from_bytes_unchecked(&bytes)? };
        Ok(Self {
            is_valid: inner.built_time == COMPILED_TIME,
            inner,
        })
    }

    pub fn update(&mut self, path: &Path) -> anyhow::Result<()> {
        if self.is_valid {
            return Ok(());
        }
        if path.exists() {
            fs::remove_file(path)?;
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.inner.built_time = COMPILED_TIME;
        let bytes = rkyv::to_bytes::<_, 0>(&self.inner)?;
        fs::write(path, bytes)?;

        Ok(())
    }
}

#[derive(Archive, Deserialize, Serialize, Default)]
#[archive()]
pub struct Inner {
    pub built_time: [u8; 16],

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
    pub stem: Option<String>,
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
