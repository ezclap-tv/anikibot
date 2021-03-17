#[macro_export]
macro_rules! thread_log {
    ($name:ident!, $fmt:tt, $( $frag:expr ),* ) => {
        log::$name!(
            "[IN THREAD {}] {}",
            std::thread::current().name().unwrap_or("<NO-NAME>"),
            format!($fmt, $( $frag ),*)
        );
    };
    ($name:ident!, $fmt:tt) => {
        log::$name!(
            "[IN THREAD {}] {}",
            std::thread::current().name().unwrap_or("<NO-NAME>"),
            $fmt,
        );
    };
}

#[macro_export]
macro_rules! thread_error {
    ($fmt:tt, $( $frag:expr ),* ) => {
        thread_log!(error!, $fmt, $( $frag ),*);
    };
    ($fmt:tt) => {
        thread_log!(error!, $fmt);
    };
}

#[macro_export]
macro_rules! thread_info {
    ($fmt:tt, $( $frag:expr ),* ) => {
        thread_log!(info!, $fmt, $( $frag ),*);
    };
    ($fmt:tt) => {
        thread_log!(info!, $fmt);
    };
}

#[macro_export]
macro_rules! thread_try {
    ($expr:expr, $fmt:tt $( , )? $( $frag:expr ),*  ) => {
        match $expr {
            Ok(ok) => ok,
            Err(e) => {
                thread_error!($fmt, $( $frag ),* e);
                return;
            }
        }
    };
}
