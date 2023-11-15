macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
}
pub(crate) use expand_value;
