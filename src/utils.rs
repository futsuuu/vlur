use std::ffi::OsStr;

macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
}
pub(crate) use expand_value;

#[cfg(unix)]
pub fn os_str_as_bytes<'a>(os_str: &'a OsStr) -> &'a [u8] {
    use std::os::unix::ffi::OsStrExt;
    os_str.as_bytes()
}

#[cfg(windows)]
pub fn os_str_as_bytes<'a>(os_str: &'a OsStr) -> &'a [u8] {
    os_str.as_encoded_bytes()
}
