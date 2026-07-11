# ADR 0001: WebTransport Stack

## Status

Accepted

## Decision

Use `wtransport` 0.7.1 on Quinn, rustls, and Tokio for the native client core.
Keep the C ABI independent from crate-specific types so the protocol stack can
be replaced without changing the Godot-facing ABI.

## Consequences

The stack supports WebTransport sessions, streams, datagrams, native roots, and
browser-compatible certificate hashes. It does not provide HTTP/3 connection
pooling or multiple WebTransport sessions on one connection. Those capabilities
remain explicitly unsupported until a demonstrated product requirement justifies
moving to lower-level `h3` integration.
