use std::path::PathBuf;

use mlua::{
    ChunkMode, FromLuaMulti, Function, Integer, IntoLua, IntoLuaMulti, Lua, Result,
    Table,
};

pub struct Nvim<'lua> {
    lua: &'lua Lua,
    raw: Table<'lua>,
    cache: Cache<'lua>,
}

#[derive(Default)]
struct Cache<'lua> {
    plugin_loaders: Option<Table<'lua>>,
    set_opt: Option<Function<'lua>>,
    get_opt: Option<Function<'lua>>,
    exec: Option<Function<'lua>>,
    cache_dir: Option<String>,
    create_autocmd: Option<Function<'lua>>,
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
    pub fn new(lua: &'lua Lua) -> Result<Nvim<'lua>> {
        let loader = lua.create_function(|lua, ()| {
            lua.load(&include_bytes!(concat!(env!("OUT_DIR"), "/nvim.luac"))[..])
                .set_mode(ChunkMode::Binary)
                .eval::<Table>()
        })?;
        let raw = lua.load_from_function("vlur.nvim", loader)?;
        let r = Self {
            lua,
            raw,
            cache: Cache::default(),
        };
        Ok(r)
    }

    pub fn set_plugin_loader(
        &mut self,
        name: &str,
        loader: Function<'lua>,
    ) -> Result<()> {
        cache!(self.plugin_loaders).raw_set(name, loader)
    }

    pub fn get_plugin_loader(&mut self, name: &str) -> Result<Function<'lua>> {
        cache!(self.plugin_loaders).raw_get(name)
    }

    pub fn set_opt<A>(&mut self, name: &str, value: A) -> Result<()>
    where
        A: IntoLuaMulti<'lua>,
    {
        cache!(self.set_opt).call((name, value))
    }

    pub fn get_opt<R>(&mut self, name: &str) -> Result<R>
    where
        R: FromLuaMulti<'lua>,
    {
        cache!(self.get_opt).call(name)
    }

    pub fn exec<A>(&mut self, script: A) -> Result<()>
    where
        A: IntoLuaMulti<'lua>,
    {
        cache!(self.exec).call(script)
    }

    pub fn cache_dir(&mut self) -> Result<PathBuf> {
        Ok(PathBuf::from(cache!(self.cache_dir)))
    }

    pub fn create_autocmd<E, P>(
        &mut self,
        event: E,
        pattern: P,
        callback: Function<'lua>,
        once: bool,
    ) -> Result<Integer>
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
}
