macro_rules! unwrap_or_exit {
    ($e:expr) => {{
        use std::process;
        use tracing::error;
        ($e).map_err(|e| {
            error!("{}", e);
            eprintln!("error: {}", e);
            process::exit(1);
        })
        .unwrap()
    }};
}
