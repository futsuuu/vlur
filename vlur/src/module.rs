use mlua::prelude::*;

use crate::{install::installers, lazy::handlers as lazy_handlers, setup::setup};

pub struct Module;

impl Module {
    pub fn new() -> Self {
        Self
    }
}

impl LuaUserData for Module {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_function_get("lazy", |lua, _| lazy_handlers(lua));
        fields.add_field_function_get("install", |lua, _| installers(lua));
        fields.add_field_function_get("setup", |lua, _| lua.create_function(setup));

        log::trace!("loaded the Rust module");
    }
}
