use mlua::{Lua, Table};

#[mlua::lua_module]
fn vlur(lua: &Lua) -> mlua::Result<Table> {
    let exports = lua.create_table()?;
    #[cfg(debug_assertions)]
    exports.set("debug", true)?;

    exports.set("hello", "world")?;
    Ok(exports)
}
