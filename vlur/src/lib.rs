mod cache;
mod install;
mod lazy;
mod module;
mod nvim;
mod plugin;
mod runtimepath;
mod setup;
mod ui;
mod utils;

#[mlua::lua_module]
fn vlur(_lua: &mlua::Lua) -> mlua::Result<module::Module> {
    utils::setup_logger().or(Err(mlua::Error::runtime("Failed to setup logger")))?;

    Ok(module::Module::new())
}
