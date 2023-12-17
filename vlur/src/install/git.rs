use std::{
    io,
    path::PathBuf,
    process::Command,
    thread::{self, JoinHandle},
};

use mlua::prelude::*;

use crate::ui::Progress;

pub struct Git {
    url: String,
    path: Option<PathBuf>,
    thread: Option<JoinHandle<io::Result<()>>>,
}

impl LuaUserData for Git {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("setup", |_lua, this, path| this.setup(path));
        methods.add_method_mut("install", |_lua, this, _: ()| this.install());
        methods.add_method_mut("progress", |_lua, this, _: ()| this.progress());
    }
}

impl<'lua> Git {
    pub fn new(_lua: &'lua Lua, url: LuaString<'lua>) -> LuaResult<Self> {
        let url = url.to_str()?.to_string();
        Ok(Self {
            url,
            path: None,
            thread: None,
        })
    }

    fn setup(&mut self, path: LuaString<'lua>) -> LuaResult<bool> {
        debug_assert!(self.path.is_none());
        debug_assert!(self.thread.is_none());

        let path = PathBuf::from(path.to_str()?.to_string());
        let result = path.exists();
        self.path = Some(path);

        Ok(result)
    }

    fn install(&mut self) -> LuaResult<()> {
        if self.thread.is_some() {
            return Ok(());
        }

        let url = self.url.as_str();
        let path = self.path.as_ref().unwrap();
        let mut cmd = Command::new("git");
        cmd.args(["clone", url]).arg(path.as_os_str());

        let thread = thread::spawn(move || {
            cmd.status()?;
            Ok(())
        });
        self.thread = Some(thread);

        Ok(())
    }

    fn progress(&mut self) -> LuaResult<Progress> {
        let is_finished = match self.thread {
            Some(ref thread) => thread.is_finished(),
            None => true,
        };
        if is_finished {
            self.thread = None;
        }
        Ok(Progress { is_finished })
    }
}
