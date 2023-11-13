use std::{
    env, fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let built_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    fs::write(out_dir.join("built_time"), built_time.to_string()).unwrap();
}
