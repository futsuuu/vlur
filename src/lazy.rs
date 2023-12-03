mod event;

use mlua::prelude::*;

use crate::utils::expand_value;

const REG_NAME: &str = concat!(env!("CARGO_PKG_NAME"), ".stop_funcs");

fn get_registry(lua: &Lua) -> LuaResult<LuaTable<'_>> {
    let r = lua
        .named_registry_value(REG_NAME)
        .unwrap_or(lua.create_table()?);
    Ok(r)
}

pub struct Handler<'lua> {
    handler: LuaValue<'lua>,
    lua_start: LuaFunction<'lua>,
    lua_stop: LuaFunction<'lua>,
}

impl<'lua> Handler<'lua> {
    pub fn start(
        &self,
        lua: &'lua Lua,
        plugin_id: LuaString<'lua>,
        plugin_loader: LuaFunction<'lua>,
    ) -> LuaResult<()> {
        let funcs = get_registry(lua)?;
        funcs
            .get(plugin_id.clone())
            .unwrap_or(lua.create_table()?)
            .push(self.lua_stop.clone())?;
        lua.set_named_registry_value(REG_NAME, funcs)?;

        self.lua_start.call((self.handler.clone(), plugin_loader))?;

        Ok(())
    }
}

impl<'lua> IntoLua<'lua> for Handler<'lua> {
    fn into_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(self.handler)
    }
}

impl<'lua> FromLua<'lua> for Handler<'lua> {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        let (lua_start, lua_stop) = match value {
            LuaValue::Table(ref t) => {
                expand_value!(t, {
                    start: LuaFunction,
                    stop: LuaFunction,
                });
                (start, stop)
            }
            LuaValue::UserData(ref ud) => {
                expand_value!(ud, {
                    start: LuaFunction,
                    stop: LuaFunction,
                });
                (start, stop)
            }
            _ => {
                let error = LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "table or userdata",
                    message: None,
                };
                return Err(error);
            }
        };

        let handler = Self {
            handler: value,
            lua_start,
            lua_stop,
        };

        Ok(handler)
    }
}

pub fn handlers(lua: &Lua) -> LuaResult<LuaTable<'_>> {
    let t = lua.create_table()?;

    t.set("event", lua.create_function(event::Event::new)?)?;

    Ok(t)
}
