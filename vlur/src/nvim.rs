use std::{env, path::{PathBuf, Path}};

use log::trace;
use mlua::{prelude::*, ChunkMode};

use crate::utils::expand_value;

const REGISTRY_KEY: &str = concat!(env!("CARGO_PKG_NAME"), ".nvim_api");
/// Separator character used for Neovim options.
pub const OPT_SEP: char = ',';

#[inline]
fn load_from_lua<'lua, R: FromLua<'lua>>(lua: &'lua Lua, name: &str) -> LuaResult<R> {
    if let Ok(t) = lua.named_registry_value::<LuaTable>(REGISTRY_KEY) {
        return t.raw_get(name);
    }
    trace!("load Neovim APIs");
    let t = lua
        .load(&include_bytes!(concat!(env!("OUT_DIR"), "/nvim.luac"))[..])
        .set_mode(ChunkMode::Binary)
        .eval::<LuaTable>()?;
    let r = t.raw_get(name);
    lua.set_named_registry_value(REGISTRY_KEY, t)?;
    r
}

macro_rules! nvim {
    ($lua:ident . $name:ident : $ty:ty) => {
        self::load_from_lua::<$ty>($lua, stringify!($name))
    };
    ($lua:ident . $name:ident ( $a:expr ) -> $ty:ty) => {{
        let f = nvim!($lua . $name : ::mlua::Function);
        let r = f.and_then(|f| f.call::<_, $ty>($a));
        r
    }};
    ($lua:ident . $name:ident ( $( $a:expr ),+ $(,)? ) -> $ty:ty) => {{
        let f = nvim!($lua . $name : ::mlua::Function);
        let r = f.and_then(|f| f.call::<_, $ty>(( $($a,)+ )));
        r
    }};
    ($lua:ident . $name:ident ( $a:expr )) => {
        nvim!($lua . $name ($a) -> ())
    };
    ($lua:ident . $name:ident ( $( $a:expr ),+ $(,)? )) => {
        nvim!($lua . $name ( $($a,)+ ) -> ())
    };
}

pub fn vimruntime() -> PathBuf {
    let var = env::var_os("VIMRUNTIME").unwrap();
    PathBuf::from(var)
}

pub fn set_opt<'lua, A>(lua: &'lua Lua, name: &str, value: A) -> LuaResult<()>
where
    A: IntoLuaMulti<'lua>,
{
    nvim!(lua.set_opt(name, value))
}

pub fn get_opt<'lua, R>(lua: &'lua Lua, name: &str) -> LuaResult<R>
where
    R: FromLuaMulti<'lua>,
{
    nvim!(lua.get_opt(name) -> R)
}

pub fn exec<'lua, A>(lua: &'lua Lua, script: A) -> LuaResult<()>
where
    A: IntoLuaMulti<'lua>,
{
    nvim!(lua.exec(script))
}

pub fn cache_dir(lua: &Lua) -> LuaResult<PathBuf> {
    Ok(Path::new(&nvim!(lua.cache_dir: String)?).join("vlur"))
}

pub fn create_autocmd<'lua, E, P>(
    lua: &'lua Lua,
    event: E,
    pattern: P,
    callback: LuaFunction<'lua>,
    once: bool,
) -> LuaResult<LuaInteger>
where
    E: IntoLua<'lua>,
    P: IntoLua<'lua>,
{
    nvim!(lua.create_autocmd(
        event.into_lua(lua)?,
        pattern.into_lua(lua)?,
        callback,
        once,
    ) -> LuaInteger)
}

pub fn del_autocmd(lua: &Lua, id: LuaInteger) -> LuaResult<()> {
    nvim!(lua.del_autocmd(id))
}

pub fn get_autocmds<'lua, E>(
    lua: &'lua Lua,
    event: E,
) -> LuaResult<LuaTableSequence<'lua, AutoCommand<'lua>>>
where
    E: IntoLua<'lua>,
{
    let event = event.into_lua(lua)?;
    let table = nvim!(lua.get_autocmds(event) -> LuaTable)?;
    Ok(table.sequence_values())
}

pub fn exec_autocmds<'lua, E>(
    lua: &'lua Lua,
    event: E,
    group: Option<LuaInteger>,
    data: LuaValue,
) -> LuaResult<()>
where
    E: IntoLua<'lua>,
{
    let event = event.into_lua(lua)?;
    nvim!(lua.exec_autocmds(event, group, data))
}

#[derive(PartialEq)]
pub struct AutoCommand<'lua> {
    pub id: Option<LuaInteger>,
    pub group: Option<LuaInteger>,
    pub callback: LuaValue<'lua>,
}

impl<'lua> FromLua<'lua> for AutoCommand<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let value = LuaTable::from_lua(value, lua)?;
        expand_value!(value, {
            id: Option<LuaInteger>,
            group: Option<LuaInteger>,
            command: mlua::String,
            callback: Option<LuaValue>,
        });

        let autocmd = Self {
            id,
            group,
            callback: callback.unwrap_or(command.into_lua(lua)?),
        };

        Ok(autocmd)
    }
}
