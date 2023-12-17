use vlur_tests::run;

#[test]
fn default_plugins() {
    run("load_default_plugins.lua");
    run("disable_default_plugins.lua");
}

#[test]
fn packpath() {
    run("packpath/read.lua");
}

#[test]
fn install() {
    run("install.lua");
}
