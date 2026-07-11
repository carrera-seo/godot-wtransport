# Operations and Diagnostics

## Statistics

`WebTransportClient.get_connection_stats()` is a cheap snapshot. Counters are
monotonic for the client lifetime; active counts are instantaneous. The byte
counters cover application stream payload accepted by completed writes and
delivered receive reads, not QUIC or HTTP/3 overhead.

Per-session diagnostics expose RTT and maximum datagram size. Congestion
window, packet-loss, acknowledgement, minimum RTT, RTT variation, and estimated
send-rate values are deliberately absent because `wtransport` 0.7.1 does not
expose its underlying Quinn connection statistics. Zero is used when a path
value is not yet available.

## Graceful drain

Call `session.drain(timeout_ms, code, reason)` before application shutdown. The
session immediately rejects new datagrams and stream creation. Existing streams
remain writable until they finish or the deadline closes the connection. A
second drain call is rejected. The `draining_started` signal confirms that the
state transition reached Godot's main thread.

## Safe trace events

Tracing is disabled by default. When enabled, trace events share the bounded
event queue and are dropped instead of blocking transport work. The
`dropped_trace_events` counter reports this pressure.

The trace schema is an allowlist:

- `name`: fixed implementation event name;
- `session_handle`: opaque numeric handle;
- `stream_handle`: opaque numeric handle;
- `value`: byte count, timeout, close code, or RTT depending on the event.

Trace events never contain URLs, DNS names, headers, certificates, close
reasons, datagram contents, or stream contents.
