use compiled_time::bytes;

#[test]
fn test() {
    assert_ne!(bytes!(), bytes!());
}
