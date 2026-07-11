# Troubleshooting

## Unknown issuer

The server certificate does not chain to a trusted root. Use a publicly trusted
certificate, install the development CA in the operating system, or provide the
CA through `custom_ca_pem`.

## Certificate not valid for name

The URL hostname or IP address is missing from the certificate Subject
Alternative Name extension. Regenerate the certificate for the exact URL.

## Expired or not yet valid

Check both endpoint clocks and certificate dates. Certificate hashes do not
bypass validity checks.

## Connection timeout

Confirm that UDP traffic reaches the HTTP/3 endpoint and that DNS resolves to
the expected address. A TCP-only HTTPS check does not prove QUIC reachability.

## Localhost resolves to the wrong family

If the server binds only IPv4, test with `https://127.0.0.1:PORT`. If it binds
only IPv6, use `https://[::1]:PORT`. The certificate must cover the selected
address unless certificate-hash verification is used.
