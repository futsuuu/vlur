use compiled_time::bytes;

#[test]
fn test() {
    let _: [u8; 16] = bytes!();
    assert_ne!(bytes!(), bytes!());
}
