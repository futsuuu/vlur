use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn default_plugins() {
    run("tests/load_default_plugins.lua");
    run("tests/disable_default_plugins.lua");
}

fn tmp_dir() -> PathBuf {
    PathBuf::from(file!()).parent().unwrap().join("temp")
}

fn run<P: AsRef<Path>>(luafile: P) {
    let luafile = luafile.as_ref();
    let tmp_dir = tmp_dir().join(luafile.file_stem().unwrap());
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap();
    }

    let config_dir = tmp_dir.join("config");
    fs::create_dir_all(&config_dir.join("nvim")).unwrap();
    fs::copy(luafile, config_dir.join("nvim").join("init.lua")).unwrap();

    let mut cmd = Command::new("nvim");
    cmd.args(["--headless", "--cmd", "set rtp^=.", "+qa!"])
        .env("NVIM_APPNAME", "nvim")
        .env("XDG_CONFIG_HOME", config_dir)
        .env("XDG_DATA_HOME", tmp_dir.join("data"))
        .env("XDG_STATE_HOME", tmp_dir.join("state"))
        .env("XDG_CACHE_HOME", tmp_dir.join("cache"))
        .env("XDG_RUNTIME_DIR", tmp_dir.join("run"))
        .env("NVIM_LOG_FILE", tmp_dir.join("log").join("log"));

    assert!(cmd.status().unwrap().success());
}
