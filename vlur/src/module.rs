use std::sync::mpsc::Receiver;

use mlua::prelude::*;

use crate::{install::installers, lazy::handlers as lazy_handlers, setup::setup};

pub struct Module {
    log_receiver: Receiver<String>,
}

impl LuaUserData for Module {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        log::trace!("add fields");
        fields.add_field_function_get("lazy", |lua, _| lazy_handlers(lua));
        fields.add_field_function_get("install", |lua, _| installers(lua));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        log::trace!("add methods");
        methods.add_function("setup", setup);
        methods.add_method("get_log", |_lua, this, _: ()| this.get_log());
    }
}

impl Module {
    pub fn new(log_receiver: Receiver<String>) -> Self {
        Self { log_receiver }
    }

    fn get_log(&self) -> LuaResult<Option<String>> {
        Ok(self.log_receiver.try_recv().ok())
    }
}
