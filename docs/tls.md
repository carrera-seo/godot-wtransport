# TLS Configuration

WebTransport over HTTP/3 always uses TLS 1.3. Certificate verification is
enabled by default and cannot be disabled in a production build.

## Native roots

Create `WebTransportTlsOptions` without custom fields, or omit it entirely.
The Rust core loads the operating-system trust store and validates the
certificate chain, validity period, hostname, SNI, and HTTP/3 ALPN.

## Custom CA

Set `custom_ca_pem` to one or more PEM-encoded CA certificates. Custom roots
replace native roots for that connection. The normal Web PKI validity and
hostname checks remain active.

```gdscript
var options := WebTransportTlsOptions.new()
options.custom_ca_pem = FileAccess.get_file_as_bytes("res://certificates/dev-ca.pem")
client.connect_to_url("https://localhost:4433/game", options)
```

## Server certificate hashes

Add 32-byte SHA-256 certificate hashes for browser-compatible short-lived
self-signed certificates. The certificate must use ECDSA P-256, be currently
valid, and have a validity period no longer than two weeks.

## Insecure test mode

Certificate bypass is absent from normal builds. It is only compiled when both
conditions are true:

- CMake is configured with `CMAKE_BUILD_TYPE=Debug`.
- `GWT_ENABLE_INSECURE_TESTING=ON` is explicitly supplied.

CMake rejects the option for non-Debug builds. Release CI additionally verifies
that the insecure C ABI symbol is not exported.
