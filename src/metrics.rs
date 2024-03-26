use color_eyre::eyre::Result;
use maxminddb::geoip2;
use prometheus::{IntCounterVec, IntGaugeVec, Opts, Registry};
use time::OffsetDateTime;
use tracing::{debug, trace};

use crate::wireguard::WireguardState;

pub struct Metrics {
    interfaces_total: IntGaugeVec,
    peers_total: IntGaugeVec,
    peer_endpoint: IntGaugeVec,
    bytes_total: IntCounterVec,
    peer_bytes_total: IntCounterVec,
    duration_since_latest_handshake: IntGaugeVec,
}

impl Metrics {
    pub fn new(
        r: &Registry,
        maxminddb_reader: &Option<maxminddb::Reader<Vec<u8>>>,
    ) -> Result<Self> {
        trace!("Metrics::new");

        let interfaces_total = IntGaugeVec::new(
            Opts::new("wireguard_interfaces_total", "Total number of interfaces"),
            &[],
        )?;

        let peers_total = IntGaugeVec::new(
            Opts::new(
                "wireguard_peers_total",
                "Total number of peers per interfaces",
            ),
            &["interface"],
        )?;

        let peer_endpoint = IntGaugeVec::new(
            Opts::new("wireguard_peer_endpoint", "Peers info. static value"),
            match maxminddb_reader {
                Some(_) => &[
                    "interface",
                    "endpoint_ip",
                    "endpoint_country",
                    "peer",
                    "alias",
                ],
                None => &["interface", "endpoint_ip", "peer", "alias"],
            },
        )?;

        let bytes_total = IntCounterVec::new(
            Opts::new(
                "wireguard_bytes_total",
                "Total number of bytes per direction per interface",
            ),
            &["interface", "direction"],
        )?;

        let peer_bytes_total = IntCounterVec::new(
            Opts::new(
                "wireguard_peer_bytes_total",
                "Total number of bytes per direction for a peer",
            ),
            &["interface", "peer", "direction", "alias"],
        )?;

        let duration_since_latest_handshake = IntGaugeVec::new(
            Opts::new(
                "wireguard_duration_since_latest_handshake",
                "Duration in milliseconds since latest handshake for a peer",
            ),
            &["interface", "peer", "alias"],
        )?;

        debug!("Registering wireguard metrics");
        r.register(Box::new(interfaces_total.clone()))?;
        r.register(Box::new(peers_total.clone()))?;
        r.register(Box::new(peer_endpoint.clone()))?;
        r.register(Box::new(bytes_total.clone()))?;
        r.register(Box::new(peer_bytes_total.clone()))?;
        r.register(Box::new(duration_since_latest_handshake.clone()))?;

        Ok(Self {
            interfaces_total,
            bytes_total,
            peers_total,
            peer_endpoint,
            peer_bytes_total,
            duration_since_latest_handshake,
        })
    }

    pub async fn update(
        &mut self,
        state: &WireguardState,
        maxminddb_reader: &Option<maxminddb::Reader<Vec<u8>>>,
    ) {
        let it = self.interfaces_total.with_label_values(&[]);
        it.set(state.interfaces.len() as i64);

        for (i, iface) in state.interfaces.iter().enumerate() {
            let pt = self.peers_total.with_label_values(&[iface]);
            pt.set(
                state
                    .peers
                    .iter()
                    .filter(|peer| peer.interface == i)
                    .count() as i64,
            );

            let btt = self.bytes_total.with_label_values(&[iface, "tx"]);
            let diff = state
                .peers
                .iter()
                .filter(|peer| peer.interface == i)
                .map(|peer| peer.tx_bytes)
                .sum::<u64>()
                - btt.get();
            btt.inc_by(diff);

            let btr = self.bytes_total.with_label_values(&[iface, "rx"]);
            let diff = state
                .peers
                .iter()
                .filter(|peer| peer.interface == i)
                .map(|peer| peer.rx_bytes)
                .sum::<u64>()
                - btr.get();
            btr.inc_by(diff);
        }

        for p in &state.peers {
            assert!(p.interface < state.interfaces.len());

            if let Some(endpoint) = p.endpoint {
                match maxminddb_reader {
                    Some(reader) => {
                        let endpoint_country = match reader.lookup::<geoip2::Country>(endpoint.ip())
                        {
                            Ok(reader_response) => {
                                reader_response.country.map_or_else(String::new, |c| {
                                    c.iso_code
                                        .map(|code| code.to_string())
                                        .unwrap_or_else(String::new)
                                })
                            }
                            _ => String::new(),
                        };

                        self.peer_endpoint.with_label_values(&[
                            &state.interfaces[p.interface],
                            &endpoint.ip().to_string(),
                            &endpoint_country.to_string(),
                            &p.pubkey,
                            &p.alias
                                .as_ref()
                                .map(ToOwned::to_owned)
                                .unwrap_or_else(String::new),
                        ])
                    }
                    None => self.peer_endpoint.with_label_values(&[
                        &state.interfaces[p.interface],
                        &endpoint.ip().to_string(),
                        &p.pubkey,
                        &p.alias
                            .as_ref()
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(String::new),
                    ]),
                }
                .set(1);
            };

            let pbtt = self.peer_bytes_total.with_label_values(&[
                &state.interfaces[p.interface],
                &p.pubkey,
                "tx",
                &p.alias
                    .as_ref()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(String::new),
            ]);
            let diff = p.tx_bytes - pbtt.get();
            pbtt.inc_by(diff);

            let pbtr = self.peer_bytes_total.with_label_values(&[
                &state.interfaces[p.interface],
                &p.pubkey,
                "rx",
                &p.alias
                    .as_ref()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(String::new),
            ]);
            let diff = p.rx_bytes - pbtr.get();
            pbtr.inc_by(diff);

            if let Some(latest_hs_ts) = p.handshake_timestamp {
                let now = OffsetDateTime::now_local()
                    .expect("Failed to get local offset time")
                    .unix_timestamp();

                let elapsed = now - latest_hs_ts as i64;

                self.duration_since_latest_handshake
                    .with_label_values(&[
                        &state.interfaces[p.interface],
                        &p.pubkey,
                        &p.alias
                            .as_ref()
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(String::new),
                    ])
                    .set(elapsed * 1000);
            }
        }
    }
}
