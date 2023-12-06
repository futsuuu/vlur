use std::{
    env, fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let built_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    fs::write(out_dir.join("built_time"), built_time.to_string())?;

    println!("cargo:rerun-if-changed=lua/vlur/nvim.lua");
    println!("cargo:rerun-if-changed=scripts/bump_file.lua");
    Command::new("nvim")
        .args(["-l", "scripts/bump_file.lua"])
        .arg("lua/vlur/nvim.lua")
        .arg(out_dir.join("nvim.luac"))
        .status()?;

    Ok(())
}
