# godot-wtransport

`godot-wtransport` provides a native Godot 4 GDExtension client for
WebTransport over HTTP/3. The transport core is implemented in Rust and exposed
through a small, panic-safe C ABI to an official `godot-cpp` adapter.

Prebuilt addon archives target macOS universal, Linux x86_64, and Windows
x86_64. Building from source requires the tools below.

## Quick start

1. Extract the release archive into a Godot project so that
   `addons/godot_wtransport/godot-wtransport.gdextension` exists.
2. Download the development server artifact for the host operating system.
3. Start the local echo server:

```shell
./godot-wtransport-dev-server --listen 127.0.0.1:4433
```

4. Copy the printed SHA-256 certificate hash into
   `WebTransportTlsOptions.server_certificate_hashes`, then connect to
   `https://127.0.0.1:4433/echo`. The complete Godot flow is in
   [demo/scripts/main.gd](demo/scripts/main.gd).

## Development requirements

- Rust 1.97.0
- CMake 3.28 or newer
- A C++17 compiler
- Godot 4.6.3 for the current compatibility baseline

## Core verification

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Native extension build

The build pins the official stable `godot-cpp` 4.5 API and tests the resulting
extension on Godot 4.6.3.

```shell
cmake -S . -B build -DGWT_BUILD_EXTENSION=ON -DCMAKE_BUILD_TYPE=Debug
cmake --build build --parallel
```

Build all native artifacts for the current desktop platform and create the
addon archive with SHA-256 checksums:

```shell
./scripts/build_desktop.sh
cargo build --release -p godot-wtransport-dev-server
./scripts/package_release.sh 0.1.0
```

For offline or iterative work, pass an existing stable checkout with
`-DGWT_GODOT_CPP_SOURCE=/path/to/godot-cpp`.

Run the headless load test with an explicit writable log path:

```shell
/Applications/Godot.app/Contents/MacOS/Godot \
  --headless --path ./demo \
  --log-file /private/tmp/godot-wtransport-headless.log \
  --quit-after 5
```

TLS configuration and certificate troubleshooting are documented in
[docs/tls.md](docs/tls.md) and [docs/troubleshooting.md](docs/troubleshooting.md).
Runtime statistics, graceful draining, and safe tracing are documented in
[docs/operations.md](docs/operations.md).
Artifact layout and Asset Library requirements are documented in
[docs/distribution.md](docs/distribution.md).

## License

This project uses the [Coffee-ware License](LICENSE). Offering coffee is an
optional gesture of appreciation and is not a legal requirement. Organizations
that only permit SPDX-listed licenses should complete their normal compliance
review before adoption.

Third-party components retain their original licenses. See
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
