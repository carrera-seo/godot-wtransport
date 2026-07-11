# Compatibility

## Baseline

| Component | Supported baseline |
| --- | --- |
| Godot runtime | 4.6.3 stable |
| godot-cpp API baseline | 4.5 stable (`e83fd090`) |
| Rust | 1.97.0 stable |
| C++ | C++17 |
| WebTransport | HTTP/3 sessions, streams, and datagrams |

The native extension targets macOS arm64 and x86_64 first. Windows x86_64 and
Linux x86_64 artifacts are added by the platform packaging phase.

The extension targets the stable Godot 4.5 GDExtension API and is tested on
Godot 4.6.3. Targeting the earlier minor API preserves forward compatibility
without depending on the prerelease godot-cpp 10 branch.
