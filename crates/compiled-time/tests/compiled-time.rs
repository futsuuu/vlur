use compiled_time::bytes;

#[test]
fn generated_type() {
    let _: [u8; 16] = bytes!();
}

#[test]
fn not_equal() {
    assert_ne!(bytes!(), bytes!());
}
