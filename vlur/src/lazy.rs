mod event;

use mlua::prelude::*;

use crate::utils::expand_value;

pub fn handlers(lua: &Lua) -> LuaResult<LuaTable<'_>> {
    let t = lua.create_table()?;

    t.set("event", lua.create_function(event::Event::new)?)?;

    Ok(t)
}

pub struct Handler<'lua>(Inner<'lua>);

struct Inner<'lua> {
    value: LuaValue<'lua>,
    start: LuaFunction<'lua>,
    stop: LuaFunction<'lua>,
}

impl<'lua> Handler<'lua> {
    pub fn start(
        &mut self,
        lua: &'lua Lua,
        plugin_id: LuaString<'lua>,
        plugin_loader: LuaFunction<'lua>,
    ) -> LuaResult<()> {
        self.0.bind(lua, plugin_id, plugin_loader)?;
        self.0.start.call(())
    }
}

impl<'lua> Inner<'lua> {
    fn bind(
        &mut self,
        lua: &'lua Lua,
        plugin_id: LuaString<'lua>,
        plugin_loader: LuaFunction<'lua>,
    ) -> LuaResult<()> {
        stop_funcs::set(lua, plugin_id.clone(), self.stop.clone())?;

        let plugin_loader = lua
            .create_function(load_plugin_and_stop_handlers)?
            .bind((plugin_id.clone(), plugin_loader.clone()))?;

        self.start = self.start.bind(plugin_loader)?;

        Ok(())
    }
}

fn load_plugin_and_stop_handlers(
    lua: &Lua,
    (plugin_id, plugin_loader): (LuaString, LuaFunction),
) -> LuaResult<()> {
    plugin_loader.call(())?;

    for f in stop_funcs::get(lua, plugin_id.clone())?.sequence_values() {
        let f: LuaFunction = f?;
        f.call(())?;
    }
    stop_funcs::clear(lua, plugin_id)?;

    Ok(())
}

impl<'lua> IntoLua<'lua> for Handler<'lua> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.0.into_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for Handler<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(Self(Inner::from_lua(value, lua)?))
    }
}

impl<'lua> IntoLua<'lua> for Inner<'lua> {
    fn into_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(self.value)
    }
}

impl<'lua> FromLua<'lua> for Inner<'lua> {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        let (start, stop) = match value {
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

        let start = start.bind(value.clone())?;
        let stop = stop.bind(value.clone())?;

        let handler = Self { value, start, stop };

        Ok(handler)
    }
}

/// `registry[REGISTRY_KEY]: table<plugin_id, stop_func[]>`
mod stop_funcs {
    use mlua::prelude::*;

    const REGISTRY_KEY: &str = concat!(env!("CARGO_PKG_NAME"), ".stop_funcs");

    fn get_reg_value(lua: &Lua) -> LuaResult<LuaTable<'_>> {
        let reg_value: LuaTable = lua
            .named_registry_value(REGISTRY_KEY)
            .unwrap_or(lua.create_table()?);
        Ok(reg_value)
    }

    pub fn get<'lua>(
        lua: &'lua Lua,
        plugin_id: LuaString<'lua>,
    ) -> LuaResult<LuaTable<'lua>> {
        let reg_value = get_reg_value(lua)?;
        let funcs = reg_value.raw_get(plugin_id).unwrap_or(lua.create_table()?);

        Ok(funcs)
    }

    pub fn set(
        lua: &Lua,
        plugin_id: LuaString,
        stop_func: LuaFunction,
    ) -> LuaResult<()> {
        let reg_value = get_reg_value(lua)?;
        let funcs = reg_value
            .raw_get(plugin_id.clone())
            .unwrap_or(lua.create_table()?);

        funcs.raw_push(stop_func)?;
        reg_value.raw_set(plugin_id, funcs)?;
        lua.set_named_registry_value(REGISTRY_KEY, reg_value)?;

        Ok(())
    }

    pub fn clear(lua: &Lua, plugin_id: LuaString) -> LuaResult<()> {
        let reg_value = get_reg_value(lua)?;

        reg_value.raw_set(plugin_id, LuaValue::Nil)?;
        lua.set_named_registry_value(REGISTRY_KEY, reg_value)?;

        Ok(())
    }
}
