# godot-wtransport

`godot-wtransport` provides a native Godot 4 GDExtension client for
WebTransport over HTTP/3. The transport core is implemented in Rust and exposed
through a small, panic-safe C ABI to an official `godot-cpp` adapter.

## WebTransport specification baseline

This extension was implemented and reviewed against the following explicitly
versioned WebTransport specifications:

- [W3C WebTransport Working Draft, 6 July 2026](https://www.w3.org/TR/2026/WD-webtransport-20260706/)
  for the client-facing session, datagram, stream, close, draining, and server
  certificate hash semantics.
- [IETF WebTransport over HTTP/3, draft-ietf-webtrans-http3-16, 6 July 2026](https://www.ietf.org/archive/id/draft-ietf-webtrans-http3-16.html)
  for the HTTP/3 protocol mapping, session establishment, bidirectional and
  unidirectional streams, datagrams, and session termination.

Both documents are works in progress rather than final standards. The pinned
links above identify the review baseline even when newer drafts are published.
The implementation also relies on the normative transport specifications
referenced by the IETF draft, including QUIC v1, HTTP/3, QUIC DATAGRAM, HTTP
Datagrams, and TLS 1.3.

Prebuilt addon archives target macOS universal, Linux x86_64, and Windows
x86_64. Building from source requires the tools below.

## Build and test status

The following status reflects Godot 4.7 validation on 2026-07-11:

| Environment | Build | Godot load | Native stream test | Notes |
| --- | --- | --- | --- | --- |
| macOS universal (arm64 and x86_64) | Passed | Passed on arm64 | Passed on arm64 | Universal debug and release libraries |
| Linux x86_64 | Passed | Passed | Passed | Ubuntu 24.04 CI |
| Windows x86_64 | Passed | Passed | Passed | Windows Server 2025 CI |
| Android arm64 | Passed | Not run | Not run | ELF architecture and release-symbol checks passed |
| Web | Export passed | Export smoke passed | Not applicable | Uses the optional JavaScriptBridge backend |
| iOS | Not implemented | Not run | Not run | Deferred platform |

The CI results for the latest `main` commit are available from
[GitHub Actions](https://github.com/carrera-seo/godot-wtransport/actions).

## Compatibility

- Godot 4.7 stable is the tested runtime and export baseline.
- The binary targets the stable Godot 4.5 GDExtension API, which Godot supports
  on later 4.x minor releases. There is no stable Godot 4.7 godot-cpp tag as of
  2026-07-11, so this project does not depend on a prerelease binding.
- Prebuilt native libraries are provided for macOS universal, Linux x86_64,
  Windows x86_64, and Android arm64.
- Godot Web exports use the optional browser bridge instead of a native binary.

## Installing the extension

1. Download the prebuilt addon archive for your platform from the
   [GitHub Releases page](https://github.com/carrera-seo/godot-wtransport/releases).
   Desktop users can choose the combined `all-desktop` archive. Building from
   source is not required when using these archives.

   | Target | Release archive |
   | --- | --- |
   | All desktop platforms | `godot-wtransport-0.1.0-all-desktop.zip` |
   | macOS universal | `godot-wtransport-0.1.0-macos-universal.zip` |
   | Linux x86_64 | `godot-wtransport-0.1.0-linux-x86_64.zip` |
   | Windows x86_64 | `godot-wtransport-0.1.0-windows-x86_64.zip` |
   | Android arm64 | `godot-wtransport-0.1.0-android-arm64.zip` |

   Each archive has a matching `.sha256` file on the
   [v0.1.0 release page](https://github.com/carrera-seo/godot-wtransport/releases/tag/v0.1.0).
2. Extract it at the root of your Godot project. The resulting layout must be:

```text
your-project/
└── addons/
    └── godot_wtransport/
        ├── godot-wtransport.gdextension
        └── bin/
            └── <platform debug and release libraries>
```

3. Restart the Godot editor. This is a GDExtension, so there is no editor plugin
   checkbox to enable. Confirm that `WebTransportClient`,
   `WebTransportSession`, `WebTransportStream`, and `WebTransportTlsOptions`
   are available to GDScript.
4. Commit the complete `addons/godot_wtransport` directory to the consuming
   project. Do not copy only the `.gdextension` descriptor.

## Connecting from GDScript

`WebTransportClient` is a `Node`. Keep it in the scene tree for as long as its
sessions are active so it can dispatch network events on the Godot main thread.

```gdscript
extends Node

var transport: WebTransportClient
var session: WebTransportSession

func _ready() -> void:
    transport = WebTransportClient.new()
    add_child(transport)
    transport.connection_succeeded.connect(_on_connected)
    transport.connection_failed.connect(_on_connection_failed)

    # Production servers with a publicly trusted certificate need no options.
    var request_handle := transport.connect_to_url("https://transport.example.com/game")
    if request_handle == 0:
        push_error("The WebTransport connection could not be queued")

func _on_connected(connected_session: WebTransportSession) -> void:
    session = connected_session
    session.datagram_received.connect(_on_datagram_received)
    session.incoming_bidirectional_stream.connect(_on_incoming_bidi)
    session.incoming_unidirectional_stream.connect(_on_incoming_uni)

    var error := session.send_datagram("hello".to_utf8_buffer())
    if error != OK:
        push_error("Datagram send failed: %s" % error_string(error))

func _on_datagram_received(data: PackedByteArray) -> void:
    print("Datagram: ", data.get_string_from_utf8())

func _on_connection_failed(error: Dictionary) -> void:
    push_error("WebTransport connection failed: %s" % error)

func _on_incoming_bidi(stream: WebTransportStream) -> void:
    stream.data_received.connect(func(data: PackedByteArray) -> void:
        print("Bidi bytes: ", data.size())
    )

func _on_incoming_uni(stream: WebTransportStream) -> void:
    stream.data_received.connect(func(data: PackedByteArray) -> void:
        print("Uni bytes: ", data.size())
    )
```

Outgoing streams open asynchronously. Connect `opened` before writing:

```gdscript
var stream := session.create_bidirectional_stream()
if stream == null:
    push_error("Could not create a stream")
    return

stream.data_received.connect(func(data: PackedByteArray) -> void:
    print("Reply: ", data.get_string_from_utf8())
)
stream.opened.connect(func() -> void:
    stream.write("stream payload".to_utf8_buffer())
    stream.finish()
)
```

Close a session explicitly when it is no longer needed:

```gdscript
session.close(0, "client shutdown")
```

Use `drain(timeout_ms, code, reason)` instead when already-open streams should
be given time to finish. See [docs/api.md](docs/api.md) for the full API and
[docs/operations.md](docs/operations.md) for statistics and tracing.

## Local development server and TLS

Download or build the development server, then start an echo endpoint:

```shell
./godot-wtransport-dev-server --listen 127.0.0.1:4433
```

The readiness JSON prints the URL and SHA-256 certificate hash. Development
certificates can be trusted by hash:

```gdscript
func decode_sha256(hex_value: String) -> PackedByteArray:
    var compact := hex_value.replace(":", "")
    assert(compact.length() == 64)

    var hash := PackedByteArray()
    for offset in range(0, compact.length(), 2):
        hash.append(compact.substr(offset, 2).hex_to_int())
    return hash

var tls := WebTransportTlsOptions.new()
tls.add_server_certificate_hash(
    decode_sha256("replace-with-the-64-character-hash-printed-by-the-server")
)
transport.connect_to_url("https://127.0.0.1:4433/echo", tls)
```

The hash must decode to exactly 32 bytes. Alternatively, load a development CA:

```gdscript
var tls := WebTransportTlsOptions.new()
tls.custom_ca_pem = FileAccess.get_file_as_bytes("res://certificates/local-ca.pem")
transport.connect_to_url("https://127.0.0.1:4433/echo", tls)
```

Never use the debug-only insecure connection mode in production. Certificate
configuration and failure diagnosis are covered in [docs/tls.md](docs/tls.md)
and [docs/troubleshooting.md](docs/troubleshooting.md).

## Development requirements

- Rust 1.97.0
- CMake 3.28 or newer
- A C++17 compiler
- Godot 4.7 stable for the current compatibility baseline

## Core verification

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Native extension build

The build pins the official stable `godot-cpp` 4.5 API and tests the resulting
extension on Godot 4.7 stable.

Godot Web export is optional and uses the browser implementation through a
separate JavaScriptBridge backend. See [docs/web-bridge.md](docs/web-bridge.md).

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
Android builds, mobile lifecycle policy, deferred iOS preparation, and store
privacy disclosures are documented in [docs/mobile.md](docs/mobile.md) and
[docs/store-privacy.md](docs/store-privacy.md).
Artifact layout and Asset Library requirements are documented in
[docs/distribution.md](docs/distribution.md).

## License

This project uses the [Coffee-ware License](LICENSE). Offering coffee is an
optional gesture of appreciation and is not a legal requirement. Organizations
that only permit SPDX-listed licenses should complete their normal compliance
review before adoption.

Third-party components retain their original licenses. See
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
