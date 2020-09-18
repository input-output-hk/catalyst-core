#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => (
        let formatted_message = format_args!($($arg)+).to_string();
        let level = $lvl;
        $crate::messages::LogMessageBuilder::<()>::default()
            .with_level(level)
            .with_message(formatted_message)
            .build()
            .log();
    )
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => (
        $crate::log!(log::Level::Error, $($arg)+)
    )
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => (
        $crate::log!(log::Level::Warn, $($arg)+)
    )
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => (
        $crate::log!(log::Level::Info, $($arg)+)
    )
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => (
        $crate::log!(log::Level::Debug, $($arg)+)
    )
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => (
        $crate::log!($log::Level::Trace, $($arg)+)
    )
}
