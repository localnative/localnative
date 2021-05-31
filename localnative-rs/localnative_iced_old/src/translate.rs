#[macro_export]
macro_rules! tr {
    ($msg:expr) => {
        crate::localization::tr_with_args($msg, None)
    };
    ($msg:expr; $args:expr ) => {{
        crate::localization::tr_with_args($msg, Some($args))
    }};
}
#[macro_export]
macro_rules! args {
    ($($key:expr => $value:expr),+ ) => {
        {
            let mut args: fluent_bundle::FluentArgs = fluent_bundle::FluentArgs::new();
            $(
                args.set($key, $value);
            )+
            args
        }
    };
}
