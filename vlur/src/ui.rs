use mlua::prelude::*;

use crate::utils::expand_value;

pub struct Progress {
    pub title: String,
    pub log: Option<String>,
    pub is_finished: bool,
}

impl<'lua> IntoLua<'lua> for Progress {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let t = lua.create_table()?;

        t.set("is_finished", self.is_finished)?;
        t.set("title", self.title)?;
        t.set("log", self.log)?;

        Ok(LuaValue::Table(t))
    }
}

impl<'lua> FromLua<'lua> for Progress {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let t = LuaTable::from_lua(value, lua)?;
        expand_value!(t, {
            title: String,
            log: Option<String>,
            is_finished: bool,
        });
        Ok(Self {
            title,
            log,
            is_finished,
        })
    }
}
