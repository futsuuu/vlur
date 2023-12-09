mod run;

use run::run;

#[test]
fn default_plugins() {
    run("tests/load_default_plugins.lua");
    run("tests/disable_default_plugins.lua");
}
