use std::{env, path::PathBuf};

use mlua::prelude::*;
use mlua::ChunkMode;

use crate::utils::expand_value;

/// Separator character used for Neovim options.
pub const OPT_SEP: char = ',';

pub fn vimruntime() -> PathBuf {
    let var = env::var_os("VIMRUNTIME").unwrap();
    PathBuf::from(var)
}

pub struct Nvim<'lua> {
    lua: &'lua Lua,
    raw: LuaTable<'lua>,
    cache: Cache<'lua>,
}

#[derive(Default)]
struct Cache<'lua> {
    set_opt: Option<LuaFunction<'lua>>,
    get_opt: Option<LuaFunction<'lua>>,
    exec: Option<LuaFunction<'lua>>,
    cache_dir: Option<String>,
    create_autocmd: Option<LuaFunction<'lua>>,
    del_autocmd: Option<LuaFunction<'lua>>,
    get_autocmds: Option<LuaFunction<'lua>>,
    exec_autocmds: Option<LuaFunction<'lua>>,
}

macro_rules! cache {
    ($nvim:ident . $name:ident) => {{
        if $nvim.cache.$name.is_none() {
            let v = $nvim.raw.raw_get(stringify!($name))?;
            $nvim.cache.$name = Some(v);
        }
        $nvim.cache.$name.as_ref().unwrap()
    }};
}

impl<'lua> Nvim<'lua> {
    pub fn new(lua: &'lua Lua) -> LuaResult<Nvim<'lua>> {
        let loader = lua.create_function(|lua, ()| {
            lua.load(&include_bytes!(concat!(env!("OUT_DIR"), "/nvim.luac"))[..])
                .set_mode(ChunkMode::Binary)
                .eval::<LuaTable>()
        })?;
        let raw = lua.load_from_function("vlur.nvim", loader)?;
        let r = Self {
            lua,
            raw,
            cache: Cache::default(),
        };
        Ok(r)
    }

    pub fn set_opt<A>(&mut self, name: &str, value: A) -> LuaResult<()>
    where
        A: IntoLuaMulti<'lua>,
    {
        cache!(self.set_opt).call((name, value))
    }

    pub fn get_opt<R>(&mut self, name: &str) -> LuaResult<R>
    where
        R: FromLuaMulti<'lua>,
    {
        cache!(self.get_opt).call(name)
    }

    pub fn exec<A>(&mut self, script: A) -> LuaResult<()>
    where
        A: IntoLuaMulti<'lua>,
    {
        cache!(self.exec).call(script)
    }

    pub fn cache_dir(&mut self) -> LuaResult<PathBuf> {
        Ok(PathBuf::from(cache!(self.cache_dir)))
    }

    pub fn create_autocmd<E, P>(
        &mut self,
        event: E,
        pattern: P,
        callback: LuaFunction<'lua>,
        once: bool,
    ) -> LuaResult<LuaInteger>
    where
        E: IntoLua<'lua>,
        P: IntoLua<'lua>,
    {
        cache!(self.create_autocmd).call((
            event.into_lua(self.lua)?,
            pattern.into_lua(self.lua)?,
            callback,
            once,
        ))
    }

    pub fn del_autocmd(&mut self, id: LuaInteger) -> LuaResult<()> {
        cache!(self.del_autocmd).call(id)
    }

    pub fn get_autocmds<E, P>(
        &mut self,
        event: E,
        pattern: P,
    ) -> LuaResult<LuaTableSequence<'lua, AutoCommand<'lua>>>
    where
        E: IntoLua<'lua>,
        P: IntoLua<'lua>,
    {
        let table: LuaTable = cache!(self.get_autocmds)
            .call((event.into_lua(self.lua)?, pattern.into_lua(self.lua)?))?;
        Ok(table.sequence_values())
    }

    pub fn exec_autocmds<E, P>(
        &mut self,
        event: E,
        pattern: P,
        group: Option<LuaInteger>,
        data: LuaValue,
    ) -> LuaResult<()>
    where
        E: IntoLua<'lua>,
        P: IntoLua<'lua>,
    {
        cache!(self.exec_autocmds).call((
            event.into_lua(self.lua)?,
            pattern.into_lua(self.lua)?,
            group,
            data,
        ))
    }
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
