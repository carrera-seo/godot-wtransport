# WebTransport Conformance Matrix

| Capability | Status | Verification |
| --- | --- | --- |
| HTTP/3 session establishment | supported | Rust/C/Godot local integration tests |
| Datagram send and receive | supported | Godot echo integration test |
| Bidirectional streams | supported | C local integration test |
| Unidirectional streams | supported | C local integration test |
| Session close code and reason | supported | Local server observes code and reason |
| Stream reset and stop-sending | partial | Receive-side reset signal; no public local reset API |
| Certificate hashes | supported | Certificate matrix integration tests |
| Custom CA | supported | Valid, unknown issuer, name mismatch, and expiry tests |
| Client statistics | supported | Rust unit and Phase 5 Godot integration tests |
| Session path diagnostics | supported | Phase 5 Godot integration test |
| Client graceful drain deadline | supported | Phase 5 Godot integration test |
| Server-initiated draining notification | unsupported | Upstream transport API does not expose it |
| Metadata-only trace events | supported | Phase 5 Godot integration test and field allowlist |
| Atomic writes | unsupported | API review; normal writes are not flow-control atomic |
| Send groups and send order | unsupported | API review; upstream scheduler control unavailable |
| Connection pooling | unsupported | Each session owns a separate QUIC endpoint/connection |
| Concurrent sessions | supported | Two-session Godot integration test |
| Detailed QUIC loss/cwnd/rate stats | unsupported | Upstream wrapper exposes RTT and datagram size only |

“Supported” means that the native API implements the capability and the listed
verification passes. “Partial” means that only the stated direction or subset
is public. Unsupported features have no placeholder API that could imply
browser-equivalent behavior.

The current W3C specification defines atomic writes for flow-control atomicity,
not packet or application-message boundaries. It also defines send groups and
send order as scheduling controls across streams and datagrams. The underlying
`wtransport` 0.7.1 API does not expose either facility, so this extension does
not emulate them with weaker semantics. See the
[W3C WebTransport specification](https://www.w3.org/TR/webtransport/).
