# Desktop Distribution

## Supported artifacts

| Platform | Architecture | Extension artifact |
| --- | --- | --- |
| macOS | arm64 and x86_64 | Universal `.dylib` |
| Linux | x86_64 GNU | `.so` |
| Windows | x86_64 MSVC | `.dll` |

Every release provides an addon archive, its SHA-256 checksum, a separate
symbol archive, and a platform-native local development server. Symbols and the
development server are not included in the addon archive.

The addon archive has `addons/` at its root and can be extracted directly into
a Godot project. The `.gdextension` descriptor selects debug or release
libraries using Godot feature tags.

## Reproducible packaging

`scripts/package_release.sh` uses normalized ZIP metadata (`zip -X`) and emits
a checksum next to the archive. `scripts/package_desktop_artifacts.sh` extracts
debug symbols before packaging platform CI artifacts.

## Godot Asset Library review

The package follows the Asset Library addon layout and contains no project
files outside `addons/godot_wtransport`. Before an Asset Library submission:

- publish the source repository and a stable tagged release;
- attach an HTTPS download URL for the exact addon ZIP;
- keep the plugin title, version, category, license notice, and support URL
  synchronized with the release;
- provide screenshots or a short demo video in the Asset Library listing;
- verify the archive on every declared Godot and operating-system version;
- state clearly that Coffee-ware is not an SPDX-listed license and may require
  an organization-specific compliance review.

The downloadable archive must not contain private keys, generated development
certificates, build directories, or symbol files.
