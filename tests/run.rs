use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn run(vimrc: &str) {
    let sandbox = create_sandbox(vimrc);

    let mut command = Command::new("nvim");
    command.args([
        "--headless",
        "-S",
        "scripts/quit.vim",
        "--cmd",
        "set rtp^=.",
        "-u",
        vimrc,
    ]);
    set_virtual_env(&mut command, &sandbox);

    let status = command.status().unwrap();
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

fn create_sandbox(name: &str) -> PathBuf {
    let path = Path::new(file!()).parent().unwrap().join("temp").join(name);
    if path.exists() {
        fs::remove_dir_all(&path)
            .expect("Failed to remove the old temporary directory.");
    }
    fs::create_dir_all(&path).expect("Failed to create a new temporary directory.");
    path
}
