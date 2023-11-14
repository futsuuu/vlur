macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
    ($gettable:expr, mut { $( $name:ident : $ty:ty ),+ $(,)? }) => (
        $(
            let mut $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
}
pub(crate) use expand_value;
