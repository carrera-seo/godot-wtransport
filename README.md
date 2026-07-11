# godot-wtransport

`godot-wtransport` provides a native Godot 4 GDExtension client for
WebTransport over HTTP/3. The transport core is implemented in Rust and exposed
through a small, panic-safe C ABI to an official `godot-cpp` adapter.

The project is under active development. The Rust core and C ABI are available;
the Godot-facing API is introduced in Phase 2.

## Development requirements

- Rust 1.97.0
- CMake 3.28 or newer
- A C++17 compiler
- Godot 4.6.3 for the current compatibility baseline

## Core verification

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Native extension build

The build pins the official stable `godot-cpp` 4.5 API and tests the resulting
extension on Godot 4.6.3.

```shell
cmake -S . -B build -DGWT_BUILD_EXTENSION=ON -DCMAKE_BUILD_TYPE=Debug
cmake --build build --parallel
```

For offline or iterative work, pass an existing stable checkout with
`-DGWT_GODOT_CPP_SOURCE=/path/to/godot-cpp`.

Run the headless load test with an explicit writable log path:

```shell
/Applications/Godot.app/Contents/MacOS/Godot \
  --headless --path ./demo \
  --log-file /private/tmp/godot-wtransport-headless.log \
  --quit-after 5
```

## License

This project uses the [Coffee-ware License](LICENSE). Offering coffee is an
optional gesture of appreciation and is not a legal requirement. Organizations
that only permit SPDX-listed licenses should complete their normal compliance
review before adoption.

Third-party components retain their original licenses. See
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
