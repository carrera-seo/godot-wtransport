# Third-Party Licenses

The project depends on third-party packages distributed under their own terms.
The authoritative dependency graph is `Cargo.lock`.

The primary runtime dependencies are:

| Component | Version baseline | License |
| --- | --- | --- |
| wtransport | 0.7.1 | MIT OR Apache-2.0 |
| Quinn | 0.11.11 | MIT OR Apache-2.0 |
| rustls | 0.23.41 | Apache-2.0 OR ISC OR MIT |
| Tokio | 1.52.3 | MIT |

Release automation must generate a complete notice bundle from the locked
dependency graph before publishing binary artifacts.
