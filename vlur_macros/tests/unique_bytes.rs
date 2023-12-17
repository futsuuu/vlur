use vlur_macros::unique_bytes;

#[test]
fn generated_type() {
    let _: [u8; 16] = unique_bytes!();
}

#[test]
fn not_equal() {
    assert_ne!(unique_bytes!(), unique_bytes!());
}
