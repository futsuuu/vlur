use indexmap::IndexMap;
use speedy::{Readable, Writable};

use crate::RuntimePath;

#[derive(Readable, Writable, Default)]
pub struct Cache<'a> {
    pub built_time: &'a str,
    #[speedy(skip)]
    pub is_valid: bool,
    pub package: Package<'a>,
    pub plugins: IndexMap<&'a str, Vec<File>>,
}

#[derive(Readable, Writable, Default)]
pub struct Package<'a> {
    // key
    pub packpath: &'a str,
    // value
    pub runtimepath: RuntimePath,
}

#[derive(Readable, Writable, Default)]
pub struct File {
    pub stem: String,
    pub loader: FileLoader,
}

#[derive(Readable, Writable)]
pub enum FileLoader {
    // `source <path>`
    Script(String),
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::Script("".into())
    }
}
