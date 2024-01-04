mod cache;
mod install;
mod lazy;
mod module;
mod plugin;
mod runtimepath;
mod setup;
mod ui;
mod utils;

#[mlua::lua_module]
fn vlur(_lua: &mlua::Lua) -> mlua::Result<module::Module> {
    let log_receiver = utils::setup_logger();
    Ok(module::Module::new(log_receiver))
}
