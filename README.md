# `wireguard_exporter`

![Rust Version][rustc-image]
[![crates.io][crate-image]][crate-link]
[![Dependency Status][deps-image]][deps-link]

An asynchronous Prometheus exporter for `wireguard`

`wireguard_exporter` runs `wg show [..]` and scrapes the output to
build Prometheus metrics. Because `wg` requires `root` privileges,
this tool must be run as `root` (or via `sudo`) or with the following
capabilities in both the ambient and bounding set:

- CAP_DAC_READ_SEARCH
- CAP_NET_ADMIN
- CAP_NET_RAW

# Metrics Provided

- Total number of bytes transferred in/out per peer
- Total number of bytes transferred in/out per interface
- Metrics with the info for each connected peer
- Time since last handshake per peer
- Scrape duration in milliseconds
- Scrape success

# Installation

`wireguard_exporter` is a single binary that must be placed somewhere in your
`$PATH`. One can either download 64-bit Linux binaries from [the Release Page](https://github.com/kbknapp/wireguard_exporter/releases)
 or one can also compile from source.

## Compile from Source

Ensure you have a [Rust toolchain installed](https://rustup.rs). Some of the
dependencies also require `gcc` to be installed.

```
$ git clone https://github.com/kbknapp/wireguard_exporter
$ cd wireguard_exporter
$ cargo build --release
$ sudo cp target/release/wireguard_exporter /usr/local/bin/
```

# Usage

## Command Line Interface

```
USAGE:
    wireguard_exporter [OPTIONS]

OPTIONS:
    -a, --alias <ALIAS>...           Add an alias for a given public key in the form of
                                     'pubkey:alias' (separate multiple with commas)
        --collect-interval <SECS>    How often metrics are gathered [default: 5]
    -h, --help                       Print help information
    -l, --listen-address <ADDR>      The listen address scraping metrics [default: 0.0.0.0]
    -p, --listen-port <PORT>         The listen port for scraping metrics [default: 9586]
    -q, --quiet                      Supress output at a level or lower. -q: INFO, -qq: WARN, -qqq:
                                     ERROR (i.e. everything)
    -v, --verbose                    Show verbose output at a level or higher. -v:  DEBUG, -vv:
                                     TRACE
    -V, --version                    Print version information
```

To run with the default options, and the binary is installed somewhere in your
`$PATH`:

```
$ sudo wireguard_exporter
```

# Prometheus Configuration

You can add the following scrape configs to Prometheus:

```yaml
scrape_configs:
  - job_name: 'wireguard'
    static_configs:
    - targets:
      - 'localhost:9586'
      - 'other_host:9586'

    relabel_configs:
    - source_labels: [ '__address__' ]
      regex: '(.*):\d+'
      target_label: instance
```

# Example Metrics

```
# HELP wireguard_bytes_total Total number of bytes per direction per interface
# TYPE wireguard_bytes_total counter
wireguard_bytes_total{direction="rx",interface="custom"} 19576636452
wireguard_bytes_total{direction="rx",interface="wg0"} 1091996152
wireguard_bytes_total{direction="tx",interface="custom"} 3919310388
wireguard_bytes_total{direction="tx",interface="wg0"} 2393043528
# HELP wireguard_duration_since_latest_handshake During since latest handshake for a peer
# TYPE wireguard_duration_since_latest_handshake gauge
wireguard_duration_since_latest_handshake{interface="custom",peer="q2JWEKWfLPU5UjG2Sq31xx2GsSjdhKNtdT/X/tFVyjs=",alias="kevin"} 51405
wireguard_duration_since_latest_handshake{interface="custom",peer="2ELWFmGnqhtRpu4r2PUKc0cw+ELtuMPLd6l0KsoCUBQ=",alias="jane"} 88405
wireguard_duration_since_latest_handshake{interface="custom",peer="duVVziZbyIiIPoRprisE69K0By198Cn8dPwY5bFecEk=",alias="robert"} 116405
wireguard_duration_since_latest_handshake{interface="custom",peer="nwj+Zw49AbYrzUAPzeRf8hhll/1dz8SjoOYZuB+JdT4="} 15296341405
wireguard_duration_since_latest_handshake{interface="custom",peer="QF01u5CZhH9+CWcVY9pbsuTu3QsTcSqFvni3VfOiL2s="} 34405
wireguard_duration_since_latest_handshake{interface="custom",peer="N5UQp3XbysLBAavUm1Cpv7xxjk99LwJD99z5//PsyCc="} 95405
wireguard_duration_since_latest_handshake{interface="custom",peer="QlgHHfYP3aMlRG7d6/Zp9IhUOLrpT5G2GIdODODaUHQ="} 10690033405
wireguard_duration_since_latest_handshake{interface="custom",peer="FtUeMGdNxgkVN0G9lpvOc5jtAQQ1m9DpvZPDCUdKBx0="} 96405
wireguard_duration_since_latest_handshake{interface="wg0",peer="bRQZOyOZUvHMhBvCWq2sXO0VsRu6Aq5LCACi/R3AJk8="} 42405
# HELP wireguard_interfaces_total Total number of interfaces
# TYPE wireguard_interfaces_total gauge
wireguard_interfaces_total 2
# HELP wireguard_peer_bytes_total Total number of bytes per direction for a peer
# TYPE wireguard_peer_bytes_total counter
wireguard_peer_bytes_total{direction="rx",interface="custom",peer=q2JWEKWfLPU5UjG2Sq31xx2GsSjdhKNtdT/X/tFVyjs="",alias="kevin"} 0
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="2ELWFmGnqhtRpu4r2PUKc0cw+ELtuMPLd6l0KsoCUBQ=",alias="jane"} 1240506784
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="duVVziZbyIiIPoRprisE69K0By198Cn8dPwY5bFecEk=",alias="robert"} 1312403276
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="nwj+Zw49AbYrzUAPzeRf8hhll/1dz8SjoOYZuB+JdT4="} 11962543712
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="QF01u5CZhH9+CWcVY9pbsuTu3QsTcSqFvni3VfOiL2s="} 0
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="N5UQp3XbysLBAavUm1Cpv7xxjk99LwJD99z5//PsyCc="} 0
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="QlgHHfYP3aMlRG7d6/Zp9IhUOLrpT5G2GIdODODaUHQ="} 353261276
wireguard_peer_bytes_total{direction="rx",interface="custom",peer="FtUeMGdNxgkVN0G9lpvOc5jtAQQ1m9DpvZPDCUdKBx0="} 2150081456
wireguard_peer_bytes_total{direction="rx",interface="wg0",peer=""} 1091996152
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="q2JWEKWfLPU5UjG2Sq31xx2GsSjdhKNtdT/X/tFVyjs=",alias="kevin"} 0
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="2ELWFmGnqhtRpu4r2PUKc0cw+ELtuMPLd6l0KsoCUBQ=",alias="jane"} 708900060
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="duVVziZbyIiIPoRprisE69K0By198Cn8dPwY5bFecEk=",alias="robert"} 714718444
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="nwj+Zw49AbYrzUAPzeRf8hhll/1dz8SjoOYZuB+JdT4="} 1171658320
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="QF01u5CZhH9+CWcVY9pbsuTu3QsTcSqFvni3VfOiL2s="} 0
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="N5UQp3XbysLBAavUm1Cpv7xxjk99LwJD99z5//PsyCc="} 0
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="QlgHHfYP3aMlRG7d6/Zp9IhUOLrpT5G2GIdODODaUHQ="} 88648
wireguard_peer_bytes_total{direction="tx",interface="custom",peer="FtUeMGdNxgkVN0G9lpvOc5jtAQQ1m9DpvZPDCUdKBx0="} 480852300
wireguard_peer_bytes_total{direction="tx",interface="wg0",peer="bRQZOyOZUvHMhBvCWq2sXO0VsRu6Aq5LCACi/R3AJk8="} 2393043528
# HELP wireguard_peer_endpoint Peers info. static value
# TYPE wireguard_peer_endpoint gauge
wireguard_peer_endpoint{alias="kevin",endpoint_ip="1.1.1.1",interface="custom",peer="q2JWEKWfLPU5UjG2Sq31xx2GsSjdhKNtdT/X/tFVyjs="} 1
wireguard_peer_endpoint{alias="jane",endpoint_ip="8.8.8.8",interface="custom",peer="2ELWFmGnqhtRpu4r2PUKc0cw+ELtuMPLd6l0KsoCUBQ="} 1
wireguard_peer_endpoint{alias="robert",endpoint_ip="127.0.0.1",interface="custom",peer="duVVziZbyIiIPoRprisE69K0By198Cn8dPwY5bFecEk="} 1
# HELP wireguard_peers_total Total number of peers per interfaces
# TYPE wireguard_peers_total gauge
wireguard_peers_total{interface="custom"} 7
wireguard_peers_total{interface="wg0"} 1
# HELP wireguard_scrape_duration_milliseconds Duration in milliseconds of the scrape
# TYPE wireguard_scrape_duration_milliseconds gauge
wireguard_scrape_duration_milliseconds 1
# HELP wireguard_scrape_success If the scrape was a success
# TYPE wireguard_scrape_success gauge
wireguard_scrape_success 1
```

# License

This crate is licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[rustc-image]: https://img.shields.io/badge/rustc-1.56+-blue.svg
[crate-image]: https://img.shields.io/crates/v/wireguard_exporter.svg
[crate-link]: https://crates.io/crates/wireguard_exporter
[deps-image]: https://deps.rs/repo/github/kbknapp/wireguard_exporter/status.svg
[deps-link]: https://deps.rs/repo/github/kbknapp/wireguard_exporter


[//]: # (Links)

