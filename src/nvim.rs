use std::path::PathBuf;

use mlua::{
    ChunkMode, FromLua, FromLuaMulti, Function, IntoLuaMulti, Lua, Result, Table,
};

pub struct Nvim<'lua> {
    raw: Table<'lua>,
    cache: Cache<'lua>,
}

#[derive(Default)]
struct Cache<'lua> {
    set_opt: Option<Function<'lua>>,
    get_opt: Option<Function<'lua>>,
    exec: Option<Function<'lua>>,
    cache_dir: Option<String>,
}

impl<'lua> FromLua<'lua> for Nvim<'lua> {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> Result<Self> {
        let r = Self {
            raw: Table::from_lua(value, lua)?,
            cache: Cache::default(),
        };
        Ok(r)
    }
}

macro_rules! cache {
    ($nvim:ident . $name:ident : $ty:ty) => {{
        if $nvim.cache.$name.is_none() {
            let v: $ty = $nvim.raw.get(stringify!($name))?;
            $nvim.cache.$name = Some(v);
        }
        $nvim.cache.$name.as_ref().unwrap()
    }};
}

impl<'lua> Nvim<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Nvim<'lua>> {
        let loader = lua.create_function(|lua, ()| {
            lua.load(&include_bytes!(concat!(env!("OUT_DIR"), "/nvim.luac"))[..])
                .set_mode(ChunkMode::Binary)
                .eval::<Table>()
        })?;
        let raw = lua.load_from_function("vlur.nvim", loader)?;
        let r = Self {
            raw,
            cache: Cache::default(),
        };
        Ok(r)
    }

    pub fn set_opt<A: IntoLuaMulti<'lua>>(
        &mut self,
        name: &str,
        value: A,
    ) -> Result<()> {
        cache!(self.set_opt: Function).call::<_, ()>((name, value))
    }

    pub fn get_opt<R: FromLuaMulti<'lua>>(&mut self, name: &str) -> Result<R> {
        cache!(self.get_opt: Function).call::<_, R>(name)
    }

    pub fn exec<A: IntoLuaMulti<'lua>>(&mut self, script: A) -> Result<()> {
        cache!(self.exec: Function).call::<_, ()>(script)
    }

    pub fn cache_dir(&mut self) -> Result<PathBuf> {
        Ok(PathBuf::from(cache!(self.cache_dir: String)))
    }
}
