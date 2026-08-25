# Shuttle

A cross-platform utility for moving files, data, and services between machines.

## Features

* **send** - Send files to another machine
* **receive** - Receive files from another machine
* **serve** - Serve files and directories over HTTP
* **tunnel** - Forward TCP connections over QUIC

## Building

Shuttle uses Rust and Nix for development.

```bash
nix develop
cargo build --release
```

Without Nix:

```bash
cargo build --release
```

Install locally:

```bash
cargo install --path .
```

## Usage

### Send

```bash
shuttle send file.iso 192.168.1.20:8080
```

### Receive

```bash
shuttle receive 0.0.0.0:8080 ~/Downloads
```

Received files are saved without overwriting existing files.

### Serve

Serve a file or directory over HTTP:

```bash
shuttle serve ./file.iso
shuttle serve ./folder --bind 0.0.0.0 --port 8080
```

For network-wide sharing:

```bash
shuttle serve ./file.iso --public
```

Protect a public share with a token:

```bash
shuttle serve ./file.iso --public --token
```

`--public` binds to all network interfaces and displays the URLs available on the machine.

For internet access, forward the selected port from your router to the machine running Shuttle.

### Tunnel

Forward TCP connections through QUIC:

```bash
shuttle tunnel 127.0.0.1:3000 192.168.1.20:9001
```

## Protocol

Shuttle uses QUIC with TLS 1.3.

* **Protocol version:** 1
* **Transport:** QUIC
* **Serialization:** Length-prefixed bincode

The protocol supports file transfers and TCP tunneling.

## Security

All Shuttle connections use TLS 1.3 through QUIC.

When exposing `serve` to the internet, use `--token` and restrict access with a firewall where possible.

## Development

```bash
nix develop
cargo check
cargo test
cargo fmt --check
cargo clippy
```

Supported platforms:

* macOS (Apple Silicon and Intel)
* Linux (ARM64 and x86_64)
