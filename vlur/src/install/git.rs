use std::{
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc::{channel, Receiver},
    thread::{self, JoinHandle},
};

use mlua::prelude::*;

use crate::ui::Progress;

pub struct Git {
    url: String,
    path: Option<PathBuf>,
    command: Option<GitCommand>,
}

struct GitCommand {
    thread: JoinHandle<io::Result<()>>,
    log_receiver: Receiver<String>,
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
            command: None,
        })
    }

    fn setup(&mut self, path: LuaString<'lua>) -> LuaResult<bool> {
        debug_assert!(self.path.is_none());
        debug_assert!(self.command.is_none());

        let path = PathBuf::from(path.to_str()?.to_string());
        let result = path.exists();
        self.path = Some(path);

        Ok(result)
    }

    fn install(&mut self) -> LuaResult<()> {
        if self.command.is_some() {
            return Ok(());
        }

        let url = self.url.as_str();
        let path = self.path.as_ref().unwrap();
        let mut cmd = Command::new("git");
        cmd.args(["clone", url]).arg(path.as_os_str());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .or(Err(LuaError::runtime("git command failed to start")))?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let (tx, rx) = channel();
        let tx1 = tx.clone();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let Ok(n) = reader.read_line(&mut line) else {
                    break;
                };
                if n == 0 {
                    break;
                }
                tx.send(line.trim().to_string()).ok();
            }
        });
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            loop {
                let mut line = String::new();
                let Ok(n) = reader.read_line(&mut line) else {
                    break;
                };
                if n == 0 {
                    break;
                }
                tx1.send(line.trim().to_string()).ok();
            }
        });

        let thread = thread::spawn(move || {
            child.wait()?;
            Ok(())
        });
        self.command = Some(GitCommand {
            thread,
            log_receiver: rx,
        });

        Ok(())
    }

    fn progress(&mut self) -> LuaResult<Progress> {
        let mut progress = Progress {
            title: String::from("git clone"),
            log: None,
            is_finished: true,
        };
        if let Some(ref command) = self.command {
            progress.is_finished = command.thread.is_finished();
            progress.log = command.log_receiver.try_iter().last();
        }
        if progress.is_finished {
            self.command = None;
        }
        Ok(progress)
    }
}
