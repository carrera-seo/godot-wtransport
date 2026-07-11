# Godot Web bridge

The optional Web bridge uses the browser-provided `WebTransport` API through
Godot's `JavaScriptBridge`. It is independent of the Rust/C++ native extension
and is not included in native addon packages.

## Supported common surface

`WebTransportWebClient`, `WebTransportWebSession`, and
`WebTransportWebStream` provide connection signals, datagrams, outgoing and
incoming unidirectional streams, bidirectional streams, stream writes and
session close. Browser Promises and Streams are consumed by JavaScript and
translated into events polled on Godot's main thread.

Stream writes preserve byte order but not application message boundaries.
Applications that require messages must add their own framing.

## Capability and security requirements

`WebTransportWebClient.is_supported()` requires both a browser WebTransport
implementation and a secure context. The server URL must use `https` and the
browser must be able to reach the server over HTTP/3.

The browser API accepts SHA-256 `serverCertificateHashes` for eligible short-
lived certificates. It does not expose native custom CA injection. The native
`custom_ca_pem` option therefore has no Web equivalent. Production browser
deployments should normally use a publicly trusted certificate.

## Error model differences

Native errors include structured transport, HTTP/3, stream, TLS, and operating
system fields when available. Browsers intentionally expose less protocol
detail. Web bridge failures use `category: "browser"` with the DOM exception
name, operation, and message. Browser messages are diagnostic text and must not
be parsed as a stable API.

An outgoing browser stream is delivered asynchronously through the
`stream_opened` signal because the browser creation methods return Promises.
This differs from the native API, which returns a stream resource immediately.

## Usage

The demo selects `WebTransportWebClient` for Web exports and the native
`WebTransportClient` everywhere else. Set `GWT_TEST_URL` and, for development
certificates, `GWT_TEST_CERTIFICATE_HASH` before starting the exported build.
For a hosted export, pass `url` and optionally `certificate_hash` in the page
query string. For example:

```text
https://game.example/index.html?url=https%3A%2F%2Ftransport.example%3A4433%2Fecho
```

Run `scripts/smoke_web_export.sh <godot>` to validate the export artifacts.

The bridge follows the W3C WebTransport working draft current on 2025-12-03
and Godot's stable JavaScriptBridge contract.

## Validation status

The Godot 4.6.3 Web export compiles and starts in a Chromium-based browser,
and capability detection and browser error delivery are exercised locally.
Chromium-based browser validation covers an HTTP/3 connection and echoed
datagram against the bundled server. The browser used for the 2026-07-11 run
did not expose the optional `draining` Promise, so the bridge feature-detects
that member while retaining session close handling. Keep browser round-trip
validation in the interop matrix; an export-only smoke is not evidence of
protocol success.
