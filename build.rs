use std::{
    env, fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use mlua::Lua;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let built_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    fs::write(out_dir.join("built_time"), built_time.to_string())?;

    let lua = Lua::new();
    let func = lua
        .load(include_str!("lua/vlur/nvim.lua"))
        .into_function()?;
    fs::write(out_dir.join("nvim.luac"), func.dump(true))?;

    Ok(())
}
