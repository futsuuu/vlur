mod cache;
mod install;
mod lazy;
mod nvim;
mod plugin;
mod runtimepath;
mod setup;
mod ui;
mod utils;

#[mlua::lua_module]
fn vlur(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let exports = lua.create_table()?;

    #[cfg(debug_assertions)]
    exports.set("debug", true)?;

    exports.set("lazy", lazy::handlers(lua)?)?;
    exports.set("install", install::installers(lua)?)?;
    exports.set("setup", lua.create_function(setup::setup)?)?;

    Ok(exports)
}
