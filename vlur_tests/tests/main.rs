use vlur_tests::test;

#[test]
fn default_plugins() {
    test("tests/load_default_plugins.lua");
    test("tests/disable_default_plugins.lua");
}

#[test]
fn packpath() {
    test("tests/packpath/read.lua");
}

#[test]
fn install() {
    test("tests/install.lua");
}
