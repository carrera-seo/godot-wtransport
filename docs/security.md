# Security Model

- TLS 1.3 is mandatory.
- Native roots are the default trust source.
- Custom roots retain hostname and validity checks.
- Certificate hashes enforce the WebTransport short-lived certificate rules.
- Private keys are used only by the local development server and are never
  included in addon artifacts.
- Logs must not contain private keys, session tickets, or application payloads.
- The release extension cannot disable certificate verification.
- Rust dependency advisories and third-party licenses are checked in CI.
- Linux CI builds the native boundary with AddressSanitizer and UBSan, then
  runs the repeated C ABI lifecycle harness with leak detection enabled.

Security-sensitive changes require focused certificate-matrix and release
symbol tests before merge.
