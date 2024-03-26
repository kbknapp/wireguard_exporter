use std::{
    env,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clap::Parser;
use color_eyre::{
    config::{HookBuilder, Theme},
    eyre::Result,
};
use maxminddb;
use prometheus::{IntGauge, Registry};
use prometheus_hyper::Server;
use tokio::time::{Duration, Instant};
use tracing::{debug, info};

#[macro_use]
mod macros;
mod cli;
mod metrics;
mod wireguard;

use cli::Args;
use metrics::Metrics;
use wireguard::WireguardState;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    HookBuilder::blank()
        .theme(Theme::new())
        .panic_section(
            "consider reporting the bug at https://github.com/kbknapp/wireguard_exporter",
        )
        .install()?;

    match args.verbose {
        0 => match args.quiet {
            0 => env::set_var("RUST_LOG", "wireguard_exporter=info"),
            1 => env::set_var("RUST_LOG", "wireguard_exporter=warn"),
            2 => env::set_var("RUST_LOG", "wireguard_exporter=error"),
            _ => env::set_var("RUST_LOG", "wireguard_exporter=off"),
        },
        1 => env::set_var("RUST_LOG", "wireguard_exporter=debug"),
        _ => env::set_var("RUST_LOG", "wireguard_exporter=trace"),
    }

    try_main(args).await
}

async fn try_main(args: Args) -> Result<()> {
    tracing_subscriber::fmt::init();

    let aliases = args.aliases();

    let maxminddb_reader = args.geoip_path.as_ref().map_or(None, |path| {
        Some(unwrap_or_exit!(maxminddb::Reader::open_readfile(path)))
    });

    let running = Arc::new(AtomicBool::new(true));

    info!("Registering metrics...");
    let registry = Arc::new(Registry::new());
    let mut metrics = unwrap_or_exit!(Metrics::new(&registry, &maxminddb_reader));
    let scrape_duration = unwrap_or_exit!(IntGauge::new(
        "wireguard_scrape_duration_milliseconds",
        "Duration in milliseconds of the scrape",
    ));

    let scrape_success = unwrap_or_exit!(IntGauge::new(
        "wireguard_scrape_success",
        "If the scrape was a success"
    ));
    debug!("Registering scrape metrics...");
    unwrap_or_exit!(registry.register(Box::new(scrape_duration.clone())));
    unwrap_or_exit!(registry.register(Box::new(scrape_success.clone())));

    info!("Spawning server...");
    tokio::spawn(Server::run(
        Arc::clone(&registry),
        SocketAddr::new(args.listen_address, args.listen_port),
        shutdown_signal(Arc::clone(&running)),
    ));

    let mut collect_int = tokio::time::interval(Duration::from_secs(args.collect_interval));
    while running.load(Ordering::Relaxed) {
        info!("Collecting metrics...");
        let before = Instant::now();

        debug!("Updating metrics...");
        metrics
            .update(&WireguardState::scrape(&aliases).await?, &maxminddb_reader)
            .await;
        let after = Instant::now();

        let elapsed = after.duration_since(before);
        scrape_duration.set((elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64) as i64);
        scrape_success.set(1);

        debug!("Sleeping...");
        collect_int.tick().await;
    }
    info!("Stopped the exporter");

    Ok(())
}

async fn shutdown_signal(running: Arc<AtomicBool>) {
    // Wait for the CTRL+C Signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    running.store(false, Ordering::Relaxed);
}
