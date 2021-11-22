use std::collections::HashMap;

use color_eyre::eyre::{Result, WrapErr};
use tokio::process::Command;

pub async fn wg_show_dump() -> Result<String> {
    String::from_utf8(
        Command::new("wg")
            .args(&["show", "all", "dump"])
            .output()
            .await
            .wrap_err("Failed to run 'wg show'")?
            .stdout,
    )
    .wrap_err("Failed wg show output to valid UTF-8")
}

#[derive(Debug)]
pub struct WireguardState {
    pub interfaces: Vec<String>,
    pub peers: Vec<Peer>,
}

impl WireguardState {
    pub async fn scrape(aliases: &HashMap<&str, &str>) -> Result<Self> {
        let mut peers = Vec::with_capacity(32); // Picked by fair dice roll
        let mut interfaces = Vec::new();

        for line in wg_show_dump().await?.lines() {
            // Peer lines:
            // INT PUBKEY PSK ENDPOINT ALLOWED-IPS HANDSHAKE TX RX KA
            //
            // Interface Lines:
            // INT PRIVKEY PUBKEY PORT FWMARK
            let mut segs = line.split('\t');
            let f1 = segs.next();
            let f2 = segs.next();
            let f3 = segs.next();
            let f4 = segs.next();
            let f5 = segs.next();
            let f6 = segs.next();
            let f7 = segs.next();
            let f8 = segs.next();
            let f9 = segs.next();

            match (f1, f2, f3, f4, f5, f6, f7, f8, f9) {
                (
                    Some(_iface),
                    Some(pubkey),
                    Some(_psk),
                    Some(_endpoint),
                    Some(_allowed_ips),
                    Some(handshake_ts),
                    Some(tx_bytes),
                    Some(rx_bytes),
                    Some(_keepalives),
                ) => {
                    let ts = handshake_ts.parse()?;
                    peers.push(Peer {
                        interface: interfaces.len() - 1,
                        alias: aliases.get(pubkey).map(|&s| s.into()),
                        pubkey: pubkey.into(),
                        handshake_timestamp: if ts == 0 { None } else { Some(ts) },
                        tx_bytes: tx_bytes.parse()?,
                        rx_bytes: rx_bytes.parse()?,
                    });
                }
                (
                    Some(iface),
                    Some(_privkey),
                    Some(_pubkey),
                    Some(_port),
                    Some(_fwmark),
                    None,
                    None,
                    None,
                    None,
                ) => interfaces.push(iface.into()),
                _ => todo!("return Err on invalid line from parse_line"),
            }
        }

        Ok(Self { interfaces, peers })
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Peer {
    pub pubkey: String,
    pub alias: Option<String>,
    pub interface: usize,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub handshake_timestamp: Option<u64>,
}
