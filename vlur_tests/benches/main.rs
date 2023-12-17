use vlur_tests::bench;

fn main() {
    println!("# without any config");
    bench("benches/raw_nvim.lua");
    println!("# setup vlur");
    bench("benches/only_setup.lua");
}
