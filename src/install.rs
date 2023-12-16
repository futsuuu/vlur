mod git;

use std::{path::Path, thread, time::Duration};

use mlua::prelude::*;

use crate::{utils::expand_value, ui::Progress};

pub fn installers(lua: &Lua) -> LuaResult<LuaTable<'_>> {
    let t = lua.create_table()?;

    t.set("git", lua.create_function(git::Git::new)?)?;

    Ok(t)
}

pub fn install(installers: Vec<Installer>, concurrency: usize) -> LuaResult<()> {
    if installers.len() == 0 {
        return Ok(());
    }

    let mut installers = installers.into_iter();
    let mut workings = Vec::with_capacity(concurrency);

    loop {
        match (installers.next(), workings.len()) {
            (None, 0) => break,
            (Some(installer), workings_count) if workings_count < concurrency => {
                installer.install()?;
                workings.push(installer);
            }
            _ => (),
        }
        for (i, installer) in workings.clone().iter().enumerate() {
            let progress = installer.progress()?;
            if progress.is_finished {
                workings.swap_remove(i);
            }
        }

        thread::sleep(Duration::from_millis(60));
    }

    Ok(())
}

#[derive(Clone)]
pub struct Installer<'lua>(Inner<'lua>);

#[derive(Clone)]
struct Inner<'lua> {
    value: LuaValue<'lua>,
    setup: LuaFunction<'lua>,
    install: LuaFunction<'lua>,
    progress: LuaFunction<'lua>,
}

impl<'lua> Installer<'lua> {
    pub fn setup(&self, path: &Path) -> LuaResult<bool> {
        self.0.setup.call(path.to_str().unwrap())
    }

    pub fn install(&self) -> LuaResult<()> {
        self.0.install.call(())
    }

    pub fn progress(&self) -> LuaResult<Progress> {
        self.0.progress.call(())
    }
}

impl<'lua> IntoLua<'lua> for Installer<'lua> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.0.into_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for Installer<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(Self(Inner::from_lua(value, lua)?))
    }
}

impl<'lua> IntoLua<'lua> for Inner<'lua> {
    fn into_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(self.value)
    }
}

impl<'lua> FromLua<'lua> for Inner<'lua> {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        let (setup, install, progress) = match value {
            LuaValue::Table(ref t) => {
                expand_value!(t, {
                    setup: LuaFunction,
                    install: LuaFunction,
                    progress: LuaFunction,
                });
                (setup, install, progress)
            }
            LuaValue::UserData(ref ud) => {
                expand_value!(ud, {
                    setup: LuaFunction,
                    install: LuaFunction,
                    progress: LuaFunction,
                });
                (setup, install, progress)
            }
            _ => {
                let error = LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "table or userdata",
                    message: None,
                };
                return Err(error);
            }
        };

        let setup = setup.bind(value.clone())?;
        let install = install.bind(value.clone())?;
        let progress = progress.bind(value.clone())?;

        let handler = Self {
            value,
            setup,
            install,
            progress,
        };

        Ok(handler)
    }
}
