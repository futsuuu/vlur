use hashbrown::HashSet;
use mlua::prelude::*;

use crate::{expand_value, Nvim};

pub struct Event {
    event: Vec<String>,
    pattern: Vec<String>,
    autocmd_id: Option<LuaInteger>,
}

impl LuaUserData for Event {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start", |lua, this, plugin_loader: LuaFunction| {
            this.start(lua, plugin_loader)
        });

        methods.add_method("stop", |lua, this, _: ()| this.stop(lua));
    }
}

impl<'lua> Event {
    pub fn new(
        _lua: &'lua Lua,
        (event, pattern): (LuaTable<'lua>, Option<LuaTable<'lua>>),
    ) -> LuaResult<Self> {
        let table_to_vec = |t: LuaTable| {
            t.sequence_values::<LuaValue>()
                .filter_map(|v| v.ok())
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        };
        let event = table_to_vec(event);
        let pattern = pattern.map(table_to_vec).unwrap_or(vec!["*".into()]);

        let r = Self {
            event,
            pattern,
            autocmd_id: None,
        };
        Ok(r)
    }

    fn start(
        &mut self,
        lua: &'lua Lua,
        plugin_loader: LuaFunction<'lua>,
    ) -> LuaResult<()> {
        if self.autocmd_id.is_some() {
            return Err(LuaError::runtime(
                "LazyHandler.start() called after starting",
            ));
        };

        let mut nvim = Nvim::new(lua)?;

        let event = self.event.as_slice();
        let pattern = self.pattern.as_slice();
        let plugin_loader = lua
            .create_function(exec_added_autocmds)?
            .bind(plugin_loader)?
            .bind(pattern)?;

        let id = nvim.create_autocmd(event, pattern, plugin_loader, true)?;
        self.autocmd_id = Some(id);

        Ok(())
    }

    fn stop(&self, lua: &'lua Lua) -> LuaResult<()> {
        let Some(id) = self.autocmd_id else {
            return Err(LuaError::runtime(
                "LazyHandler.stop() called before starting",
            ));
        };

        let mut nvim = Nvim::new(lua)?;
        nvim.del_autocmd(id)?;

        Ok(())
    }
}

fn exec_added_autocmds<'lua>(
    lua: &'lua Lua,
    (plugin_loader, pattern, ev): (LuaFunction, Vec<String>, LuaTable),
) -> LuaResult<()> {
    let mut nvim = Nvim::new(lua)?;

    expand_value!(ev, {
        event: LuaString,
        data: LuaValue,
    });
    let event = event.to_str()?;
    let pattern = pattern.as_slice();

    let mut exists_autocmds = Vec::new();
    let mut exists_ids = HashSet::new();
    let mut exists_groups = HashSet::new();
    for autocmd in nvim.get_autocmds(event, pattern)? {
        let autocmd = autocmd?;
        if let Some(id) = autocmd.id {
            exists_ids.insert(id);
        }
        if let Some(group) = autocmd.group {
            exists_groups.insert(group);
        }
        exists_autocmds.push(autocmd);
    }

    plugin_loader.call(())?;

    let mut executed_groups = HashSet::new();
    'autocmd: for autocmd in nvim.get_autocmds(event, pattern)? {
        let autocmd = autocmd?;
        if let Some(id) = autocmd.id {
            if exists_ids.contains(&id) {
                continue;
            }
        }
        if let Some(group) = autocmd.group {
            if exists_groups.contains(&group) {
                continue;
            }
            if executed_groups.contains(&group) {
                continue;
            }
            executed_groups.insert(group);
        }
        for exists in &exists_autocmds {
            if autocmd == *exists {
                continue 'autocmd;
            }
        }
        nvim.exec_autocmds(event, pattern, autocmd.group, data.clone())?;
    }

    Ok(())
}
