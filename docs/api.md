# Godot API

## WebTransportClient

Add `WebTransportClient` as a child node so its main-thread `_process` callback
can dispatch network events. `connect_to_url()` returns a numeric request handle
immediately. A successful connection emits `connection_succeeded(session)`;
failures emit `connection_failed(error)` with structured transport fields.

## WebTransportSession

Sessions send unreliable datagrams, create unidirectional and bidirectional
byte streams, and close with a 32-bit code and UTF-8 reason. Incoming datagrams
and streams are delivered through signals.

## WebTransportStream

Streams expose `write()` and `finish()` when writable. `data_received`,
`finished`, and `reset` report receive-side events. Writes are byte-oriented and
do not preserve application message boundaries.

## WebTransportTlsOptions

By default the client validates against native operating-system roots. Add one
or more 32-byte SHA-256 certificate hashes to connect securely to compatible
short-lived self-signed development certificates.
