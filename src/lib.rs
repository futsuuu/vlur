use mlua::{Lua, Table};

#[mlua::lua_module]
fn vlur(lua: &Lua) -> mlua::Result<Table> {
    let exports = lua.create_table()?;
    exports.set("hello", "world")?;
    Ok(exports)
}
