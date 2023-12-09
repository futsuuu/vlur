use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn load() {
    run("tests/load.lua").unwrap();
}

fn tmp_dir() -> io::Result<PathBuf> {
    Ok(Path::new(file!()).parent().unwrap().join("temp"))
}

fn run<P: AsRef<Path>>(luafile: P) -> io::Result<()> {
    let luafile = luafile.as_ref();
    let tmp_dir = tmp_dir()?.join(luafile.file_stem().unwrap());

    let config_dir = tmp_dir.join("config");
    fs::create_dir_all(&config_dir)?;
    fs::copy(luafile, config_dir.join("init.lua"))?;

    let status = Command::new("nvim")
        .args(["--cmd", "set rtp^=."])
        .arg("-l")
        .arg(luafile)
        .env("XDG_CONFIG_HOME", config_dir)
        .env("XDG_DATA_HOME", tmp_dir.join("data"))
        .env("XDG_STATE_HOME", tmp_dir.join("state"))
        .env("XDG_CACHE_HOME", tmp_dir.join("cache"))
        .env("XDG_RUNTIME_DIR", tmp_dir.join("run"))
        .env("NVIM_LOG_FILE", tmp_dir.join("log").join("log"))
        .status()?;
    fs::remove_dir_all(tmp_dir)?;
    assert!(status.success());
    Ok(())
}
