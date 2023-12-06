use std::{
    env,
    path::Path,
    process::Command,
};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    println!("cargo:rerun-if-changed=lua/vlur/nvim.lua");
    println!("cargo:rerun-if-changed=scripts/bump_file.lua");
    Command::new("nvim")
        .args(["-l", "scripts/bump_file.lua"])
        .arg("lua/vlur/nvim.lua")
        .arg(out_dir.join("nvim.luac"))
        .status()?;

    Ok(())
}
