# Godot API

## WebTransportClient

Add `WebTransportClient` as a child node so its main-thread `_process` callback
can dispatch network events. `connect_to_url()` returns a numeric request handle
immediately. A successful connection emits `connection_succeeded(session)`;
failures emit `connection_failed(error)` with structured transport fields.
`get_connection_stats()` returns bounded-queue, session, datagram, stream-byte,
failure, and trace-drop counters. Set `trace_enabled` to receive `trace_event`
dictionaries containing only an event name, opaque handles, and one numeric
value. URLs, certificate contents, reasons, and application payloads are never
included in trace events.

## WebTransportSession

Sessions send unreliable datagrams, create unidirectional and bidirectional
byte streams, and close with a 32-bit code and UTF-8 reason. Incoming datagrams
and streams are delivered through signals.

`get_diagnostics()` reports the state (`1` connecting, `2` connected, `3`
draining), stable connection identifier, estimated RTT in microseconds, and
current maximum outgoing datagram size. `drain(timeout_ms, code, reason)` stops
new datagrams and streams, permits already-open streams to finish, and closes
at the deadline. This is a client shutdown primitive; it is distinct from the
W3C server-initiated `draining` notification.

## WebTransportStream

Streams expose `write()` and `finish()` when writable. `data_received`,
`finished`, and `reset` report receive-side events. Writes are byte-oriented and
do not preserve application message boundaries.

Calls to `write()` are serialized per stream, but they are not W3C atomic
writes: a transport error can occur after a partial write and the call does not
reserve flow-control credit atomically. Applications must add framing when
message boundaries matter.

Native outgoing stream creation is asynchronous. Connect the stream's
`opened` signal before calling `write()` or `finish()`. Incoming stream signals
only deliver streams that are already open.

## WebTransportTlsOptions

By default the client validates against native operating-system roots. Add one
or more 32-byte SHA-256 certificate hashes to connect securely to compatible
short-lived self-signed development certificates.
