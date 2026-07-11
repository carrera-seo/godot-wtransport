# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added

- Rust WebTransport core with bounded event delivery.
- Panic-safe C ABI with explicit event ownership.
- Local development WebTransport server.
- Native Godot client, session, stream, and TLS option classes.
- Godot headless and local-server round-trip demo.
- Native-root, custom-CA, and certificate-hash TLS policies.
- Debug-only certificate bypass guard and certificate failure matrix.
- Reproducible desktop builds and packages for macOS universal, Linux x86_64,
  and Windows x86_64, with separate symbols and development servers.
- Client statistics, session path diagnostics, graceful drain deadlines, and
  bounded metadata-only trace events.
- Android arm64 build automation, deterministic mobile lifecycle/network-change
  policy, and deferred iOS static-linking and store-privacy preparation.
- Optional Godot Web JavaScriptBridge backend for datagrams and streams.
- Web export smoke CI and browser/native API difference documentation.
- Browser-compatible generated development-certificate validity.
