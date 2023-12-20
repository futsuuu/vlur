macro_rules! expand_value {
    ($gettable:expr, { $($name:ident : $ty:ty),+ $(,)? }) => (
        $(
            let $name: $ty = $gettable.get(stringify!($name))?;
        )+
    );
}
pub(crate) use expand_value;

#[cfg(not(debug_assertions))]
pub fn setup_logger() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(debug_assertions)]
pub fn setup_logger() -> anyhow::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = || {
        let micros = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as f64;
        micros / 1000.0
    };

    let start = millis();

    let logger = std::io::stdout();
    fern::Dispatch::new()
        .format(move |out, msg, rec| {
            let ms = millis() - start;
            out.finish(format_args!("{: >9.2} [{} {}] {}", ms, rec.level(), rec.target(), msg))
        })
        .level(log::LevelFilter::max())
        .chain(logger)
        .apply()?;
    Ok(())
}
