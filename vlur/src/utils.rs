use std::{
    sync::mpsc::{channel, Receiver},
    time::{SystemTime, UNIX_EPOCH},
};

macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
}
pub(crate) use expand_value;

pub fn setup_logger() -> Receiver<String> {
    let millis = || {
        let micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as f64;
        micros / 1000.0
    };

    let start = millis();

    let (tx, rx) = channel();
    fern::Dispatch::new()
        .format(move |out, msg, rec| {
            let ms = millis() - start;
            out.finish(format_args!(
                "{: >9.2} [{} {}] {}",
                ms,
                rec.level(),
                rec.target(),
                msg
            ))
        })
        .level(log::LevelFilter::max())
        .chain(tx)
        .apply()
        .unwrap();

    log::trace!("success to setup logger");

    rx
}
