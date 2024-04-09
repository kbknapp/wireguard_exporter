use std::{collections::HashMap, env, net::IpAddr, path::PathBuf, str::FromStr};

use clap::{crate_authors, Parser};

static VERSION: &str = env!("VERSION_WITH_GIT_HASH");
static AUTHORS: &str = crate_authors!();

/// A Prometheus exporter for WireGuard
#[derive(Parser)]
#[clap(author = AUTHORS, version = VERSION)]
pub struct Args {
    /// How often metrics are gathered
    #[clap(long, default_value = "5", value_name = "SECS")]
    pub collect_interval: u64,
    /// The listen port for scraping metrics
    #[clap(short = 'p', long, default_value = "9586", value_name = "PORT")]
    pub listen_port: u16,
    /// The listen address scraping metrics
    #[clap(short, long, default_value = "0.0.0.0", value_name = "ADDR")]
    pub listen_address: IpAddr,
    /// Show verbose output at a level or higher. -v:  DEBUG, -vv: TRACE
    #[clap(long, short, parse(from_occurrences))]
    pub verbose: u8,
    /// Supress output at a level or lower. -q: INFO, -qq: WARN, -qqq: ERROR (i.e. everything)
    #[clap(long, short, overrides_with = "verbose", parse(from_occurrences))]
    pub quiet: u8,
    /// Add an alias for a given public key in the form of 'pubkey:alias' (separate multiple with commas)
    #[clap(long, short, value_delimiter = ',', multiple_occurrences = true)]
    pub alias: Vec<Alias>,
    /// Do geoip lookup using Country MMDB from the PATH for 'endpoint_ip' attribute in the 'wireguard_peer_endpoint' metric and add attribute 'endpoint_country'
    #[clap(short, long, value_name = "PATH")]
    pub geoip_path: Option<PathBuf>,
}

impl Args {
    pub fn aliases(&self) -> HashMap<&str, &str> {
        let mut map = HashMap::new();
        for alias in &self.alias {
            let Alias {
                inner: (pubkey, alias),
            } = alias;
            map.insert(pubkey.as_ref(), alias.as_ref());
        }

        map
    }
}

#[derive(Clone, Debug)]
pub struct Alias {
    // A base64 encoded public key and a human readable alias
    // (pubkey, alias)
    pub inner: (String, String),
}

impl FromStr for Alias {
    type Err = String;
    fn from_str(s: &str) -> Result<Alias, Self::Err> {
        let mut parts = s.split(':');
        let pubkey = parts.next();
        let alias = parts.next();

        match (pubkey, alias) {
            (Some(pubkey), None) => Err(format!(
                "must be in the format 'PUBKEY:ALIAS' but found '{}'",
                pubkey
            )),
            (None, _) => unreachable!(),
            (Some(pubkey), Some(alias)) => {
                if pubkey.is_empty() || alias.is_empty() {
                    return Err(format!(
                        "\t\nMust be in the format 'PUBKEY:ALIAS' but found '{}:{}'",
                        pubkey, alias
                    ));
                }

                if pubkey.len() != 44 {
                    return Err(format!("\t\nPUBKEY '{}' has an invalid length", pubkey,));
                }

                if base64::decode(pubkey).is_err() {
                    return Err(format!("\n\t'{}' is not a valid public key", pubkey,));
                }

                Ok(Alias {
                    inner: (pubkey.into(), alias.into()),
                })
            }
        }
    }
}
