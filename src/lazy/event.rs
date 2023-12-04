use hashbrown::HashSet;
use mlua::prelude::*;

use crate::{nvim::Nvim, utils::expand_value};

pub struct Event {
    event: Vec<String>,
    pattern: Vec<String>,
    autocmd_ids: Vec<LuaInteger>,
}

impl LuaUserData for Event {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start", |lua, this, plugin_loader: LuaFunction| {
            this.start(lua, plugin_loader)
        });

        methods.add_method_mut("stop", |lua, this, _: ()| this.stop(lua));
    }
}

impl<'lua> Event {
    pub fn new(
        lua: &'lua Lua,
        (event, pattern): (LuaValue<'lua>, Option<LuaValue<'lua>>),
    ) -> LuaResult<Self> {
        let value_to_vec = |v: LuaValue| {
            let t = match v {
                LuaValue::Table(t) => t,
                LuaValue::String(s) => {
                    let t = lua.create_table()?;
                    t.push(s)?;
                    t
                }
                _ => {
                    let err = LuaError::FromLuaConversionError {
                        from: v.type_name(),
                        to: "string or table",
                        message: None,
                    };
                    return Err(err);
                }
            };
            Ok(t.sequence_values::<String>()
                .filter_map(|v| v.ok())
                .collect::<Vec<_>>())
        };

        let event = value_to_vec(event)?;
        let pattern = match pattern {
            Some(p) => value_to_vec(p)?,
            None => vec![String::from("*")],
        };

        let r = Self {
            event,
            pattern,
            autocmd_ids: Vec::new(),
        };
        Ok(r)
    }

    fn start(
        &mut self,
        lua: &'lua Lua,
        plugin_loader: LuaFunction<'lua>,
    ) -> LuaResult<()> {
        let mut nvim = Nvim::new(lua)?;

        let event = self.event.as_slice();
        let pattern = self.pattern.as_slice();
        let plugin_loader = lua
            .create_function(exec_added_autocmds)?
            .bind(plugin_loader)?;

        let id = nvim.create_autocmd(event, pattern, plugin_loader, true)?;
        self.autocmd_ids.push(id);

        Ok(())
    }

    fn stop(&mut self, lua: &'lua Lua) -> LuaResult<()> {
        if self.autocmd_ids.is_empty() {
            return Ok(());
        }

        let mut nvim = Nvim::new(lua)?;
        for id in &self.autocmd_ids {
            nvim.del_autocmd(*id)?;
        }
        self.autocmd_ids.clear();

        Ok(())
    }
}

fn exec_added_autocmds(
    lua: &Lua,
    (plugin_loader, ev): (LuaFunction, LuaTable),
) -> LuaResult<()> {
    let mut nvim = Nvim::new(lua)?;

    expand_value!(ev, {
        event: LuaString,
        data: LuaValue,
    });
    let event = event.to_str()?;

    let mut exists_autocmds = Vec::new();
    let mut exists_ids = HashSet::new();
    let mut exists_groups = HashSet::new();
    for autocmd in nvim.get_autocmds(event)? {
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
    'autocmd: for autocmd in nvim.get_autocmds(event)? {
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
        nvim.exec_autocmds(event, autocmd.group, data.clone())?;
    }

    Ok(())
}
