use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use StartupOption::*;

pub enum StartupOption<'a> {
    Env(&'a str, &'a str),
    Headless,
    SetRtp,
    StartupTime(&'a str),
    Quit,
    QuitWithCode,
}

pub fn run(vimrc: &str, recreate_sandbox: bool, opts: &[StartupOption]) {
    let sandbox = create_sandbox(vimrc, recreate_sandbox);

    let mut command = Command::new("nvim");
    command.args(["-u", vimrc]);
    set_virtual_env(&mut command, &sandbox);
    for opt in opts {
        match opt {
            Env(key, val) => command.env(key, val),
            Headless => command.arg("--headless"),
            SetRtp => command.args(["--cmd", "set rtp^=.."]),
            StartupTime(file) => command.args(["--startuptime", file]),
            Quit => command.args(["-c", "quitall!"]),
            QuitWithCode => command.args(["-S", "../scripts/quit.vim"]),
        };
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }
            print!("{line}");
            line.clear();
        }
    });
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }
            eprint!("{line}");
            line.clear();
        }
    });
    let status = child.wait().unwrap();
    assert!(status.success());
}

fn set_virtual_env(command: &mut Command, sandbox: &Path) {
    command
        .env("NVIM_APPNAME", "nvim")
        .env("XDG_DATA_HOME", sandbox.join("data"))
        .env("XDG_STATE_HOME", sandbox.join("state"))
        .env("XDG_CACHE_HOME", sandbox.join("cache"))
        .env("XDG_CONFIG_HOME", sandbox.join("config"))
        .env("XDG_RUNTIME_DIR", sandbox.join("run"))
        .env("NVIM_LOG_FILE", sandbox.join("log").join("log"));
}

fn create_sandbox(name: &str, remove_old: bool) -> PathBuf {
    let path = Path::new("temp").join(name);
    if remove_old && path.exists() {
        fs::remove_dir_all(&path)
            .expect("Failed to remove the old temporary directory.");
    }
    fs::create_dir_all(&path).expect("Failed to create a new temporary directory.");
    path
}
