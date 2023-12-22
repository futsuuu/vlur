use vlur_tests::{test, test_cache};

#[test]
fn setup() {
    test("tests/setup.lua");
    test_cache("tests/setup.lua");
}

#[test]
fn default_plugins() {
    test("tests/disable_default_plugins.lua");
    test_cache("tests/disable_default_plugins.lua");
}

#[test]
fn packpath() {
    test("tests/packpath/read.lua");
    test_cache("tests/packpath/read.lua");
}

#[test]
fn install() {
    test("tests/install.lua");
    test_cache("tests/install.lua");
}
