use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::RuntimePath;

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    #[serde(skip, default = "return_true")]
    pub is_valid: bool,
    pub packpath: String,
    pub package_rtp: RuntimePath,
    pub plugin_rtp: RuntimePath,
    pub load_script: String,
}

// NOTE: serde doesn't support default literals currently.
// https://github.com/serde-rs/serde/issues/368
fn return_true() -> bool {
    true
}

impl Cache {
    const NAME: &str = "cache.json";

    pub fn read(cache_dir: &Path) -> anyhow::Result<Self> {
        let raw = fs::read(cache_dir.join(Self::NAME))?;
        let deserialized: Self = serde_json::from_slice(&raw)?;
        Ok(deserialized)
    }

    pub fn write(&self, cache_dir: &Path) -> anyhow::Result<()> {
        let serialized = serde_json::to_vec(self)?;
        fs::write(cache_dir.join(Self::NAME), serialized)?;
        Ok(())
    }
}
