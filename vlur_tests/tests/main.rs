use vlur_tests::test;

#[test]
fn setup() {
    test("tests/setup.lua");
}

#[test]
fn default_plugins() {
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
