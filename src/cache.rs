use speedy::{Readable, Writable};

use crate::RuntimePath;

#[derive(Readable, Writable, Default)]
pub struct Cache {
    #[speedy(skip)]
    pub is_valid: bool,
    pub packpath: String,
    pub package_rtp: RuntimePath,
    pub plugin_rtp: RuntimePath,
    pub load_script: String,
}
