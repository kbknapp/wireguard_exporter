use std::{env, net::IpAddr};

use clap::{crate_authors, Parser};

static VERSION: &str = env!("VERSION_WITH_GIT_HASH");
static AUTHORS: &str = crate_authors!();

/// A Prometheus exporter for WireGuard
#[derive(Parser)]
#[clap(author = AUTHORS, version = VERSION)]
pub(crate) struct Args {
    /// How often metrics are gathered
    #[clap(long, default_value = "5", value_name = "SECS")]
    pub(crate) collect_interval: u64,
    /// The listen port for scraping metrics
    #[clap(short = 'p', long, default_value = "9586", value_name = "PORT")]
    pub(crate) listen_port: u16,
    /// The listen address scraping metrics
    #[clap(short, long, default_value = "0.0.0.0", value_name = "ADDR")]
    pub(crate) listen_address: IpAddr,
    /// Show verbose output at a level or higher. -v:  DEBUG, -vv: TRACE
    #[clap(long, short, parse(from_occurrences))]
    pub(crate) verbose: u8,
    /// Supress output at a level or lower. -q: INFO, -qq: WARN, -qqq: ERROR (i.e. everything)
    #[clap(long, short, overrides_with = "verbose", parse(from_occurrences))]
    pub(crate) quiet: u8,
}
