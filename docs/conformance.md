# WebTransport Conformance Matrix

| Capability | Status | Verification |
| --- | --- | --- |
| HTTP/3 session establishment | supported | Local integration test |
| Datagram send and receive | supported | Local echo integration test |
| Bidirectional streams | supported | Core API test |
| Unidirectional streams | supported | Core API test |
| Session close code and reason | supported | Core state test |
| Stream reset and stop-sending | experimental | Phase 5 test |
| Certificate hashes | supported | Phase 3 certificate matrix |
| Custom CA | planned | Phase 3 |
| Connection statistics | experimental | Phase 5 |
| Graceful draining | planned | Phase 5 |
| Atomic writes | unsupported | Under specification review |
| Send groups | unsupported | Under specification review |
| Connection pooling | unsupported | wtransport uses one QUIC connection per session |

“Supported” identifies an implemented API. “Verified” is reserved for behavior
that passes the independent browser and server interoperability matrix.
