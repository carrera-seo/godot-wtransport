# Compatibility

## Baseline

| Component | Supported baseline |
| --- | --- |
| Godot runtime | 4.7 stable |
| godot-cpp API baseline | 4.5 stable (`e83fd090`) |
| Rust | 1.97.0 stable |
| C++ | C++17 |
| WebTransport | HTTP/3 sessions, streams, and datagrams |

The native extension ships for macOS arm64 and x86_64 as a universal binary,
Windows x86_64 using the MSVC ABI, and Linux x86_64 using the GNU ABI. macOS
12.0 is the minimum deployment target.

The extension targets the stable Godot 4.5 GDExtension API and is tested on
Godot 4.7. Targeting the earlier minor API preserves forward compatibility
without depending on a prerelease godot-cpp release.

As of 2026-07-11, godot-cpp has no `godot-4.7-stable` tag and its independent
10.x line is still prerelease. The project therefore retains the official 4.5
stable binding until a stable newer binding is published.
