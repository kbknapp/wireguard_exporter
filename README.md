# `wireguard_exporter`

![Rust Version][rustc-image]
[![crates.io][crate-image]][crate-link]
[![Documentation][docs-image]][docs-link]
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
    wireguard_exporter [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Supress output at a level or lower. -q: INFO, -qq: WARN, -qqq: ERROR (i.e.
                     everything)
    -v, --verbose    Show verbose output at a level or higher. -v:  DEBUG, -vv: TRACE
    -V, --version    Prints version information

OPTIONS:
        --collect-interval <SECS>    How often metrics are gathered [default: 5]
    -l, --listen-address <ADDR>      The listen address scraping metrics [default: 0.0.0.0]
    -p, --listen-port <PORT>         The listen port for scraping metrics [default: 9455]
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
TODO
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

[rustc-image]: https://img.shields.io/badge/rustc-1.53+-blue.svg
[crate-image]: https://img.shields.io/crates/v/wireguard_exporter.svg
[crate-link]: https://crates.io/crates/wireguard_exporter
[docs-image]: https://docs.rs/wireguard_exporter/badge.svg
[docs-link]: https://docs.rs/wireguard_exporter
[deps-image]: https://deps.rs/repo/github/kbknapp/wireguard_exporter/status.svg
[deps-link]: https://deps.rs/repo/github/kbknapp/wireguard_exporter


[//]: # Links

