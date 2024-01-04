use std::{env, io, path::Path, process::Command};

fn main() -> io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    println!("cargo:rerun-if-changed=nvim.lua");
    println!("cargo:rerun-if-changed=../scripts/dump_file.lua");
    Command::new("nvim")
        .args(["-l", "../scripts/dump_file.lua"])
        .arg("nvim.lua")
        .arg(out_dir.join("nvim.luac"))
        .status()?;

    Ok(())
}
