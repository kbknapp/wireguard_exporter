use chrono::{DateTime, Local, TimeZone};
use color_eyre::eyre::Result;
use prometheus::{IntCounterVec, IntGaugeVec, Opts, Registry};
use tracing::{debug, trace};

use crate::wireguard::WireguardState;

pub struct Metrics {
    interfaces_total: IntGaugeVec,
    peers_total: IntGaugeVec,
    bytes_total: IntCounterVec,
    peer_bytes_total: IntCounterVec,
    duration_since_latest_handshake: IntGaugeVec,
}

impl Metrics {
    pub fn new(r: &Registry) -> Result<Self> {
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
            &["interface", "peer", "direction"],
        )?;

        let duration_since_latest_handshake = IntGaugeVec::new(
            Opts::new(
                "wireguard_duration_since_latest_handshake",
                "During since latest handshake for a peer",
            ),
            &["interface", "peer"],
        )?;

        debug!("Registering wireguard metrics");
        r.register(Box::new(interfaces_total.clone()))?;
        r.register(Box::new(peers_total.clone()))?;
        r.register(Box::new(bytes_total.clone()))?;
        r.register(Box::new(peer_bytes_total.clone()))?;
        r.register(Box::new(duration_since_latest_handshake.clone()))?;

        Ok(Self {
            interfaces_total,
            bytes_total,
            peers_total,
            peer_bytes_total,
            duration_since_latest_handshake,
        })
    }

    pub async fn update(&mut self, state: &WireguardState) {
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

            let pbtt = self.peer_bytes_total.with_label_values(&[
                &state.interfaces[p.interface],
                &p.pubkey,
                "tx",
            ]);
            let diff = p.tx_bytes - pbtt.get();
            pbtt.inc_by(diff);

            let pbtr = self.peer_bytes_total.with_label_values(&[
                &state.interfaces[p.interface],
                &p.pubkey,
                "rx",
            ]);
            let diff = p.rx_bytes - pbtr.get();
            pbtr.inc_by(diff);

            if let Some(latest_hs_ts) = p.handshake_timestamp {
                let now: DateTime<Local> = Local::now();
                let hs_ts: DateTime<Local> = Local.timestamp(latest_hs_ts as i64, 0);

                let elapsed = now.signed_duration_since(hs_ts);

                self.duration_since_latest_handshake
                    .with_label_values(&[&state.interfaces[p.interface], &p.pubkey])
                    .set(elapsed.num_milliseconds());
            }
        }
    }
}
