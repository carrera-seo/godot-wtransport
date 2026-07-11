# Mobile Integration

## Lifecycle policy

`WebTransportClient.close_on_application_pause` defaults to `true`. On
`NOTIFICATION_APPLICATION_PAUSED`, all active connections close immediately
with the reason `application paused`. QUIC sessions are not assumed to survive
mobile suspension. On resume, `application_resumed` is emitted and the
application creates fresh sessions.

Godot does not provide a portable notification for every Wi-Fi/cellular route
change. Platform integration should call `handle_network_change()` when its
network monitor reports a path transition. The method closes all sessions,
emits `sessions_closed_for_lifecycle` with `network_changed`, and requires the
application to reconnect. This deterministic policy avoids retaining a session
whose UDP path may no longer be valid.

## Android

Android arm64 uses NDK r29 (`29.0.14206865`), API 24 or newer, and the Rust
`aarch64-linux-android` target. Build with:

```shell
ANDROID_NDK_HOME=/path/to/android-ndk-r29 \
GWT_GODOT_CPP_SOURCE=/path/to/godot-cpp \
./scripts/build_android.sh
```

The host application must request `android.permission.INTERNET`. The extension
does not require location, advertising, device identifier, storage, camera, or
microphone permissions.

## iOS preparation status

iOS device builds use static linking for `aarch64-apple-ios`. Bitcode must not
be enabled: Xcode 15 removed bitcode support and App Store Connect no longer
accepts bitcode submissions from Xcode 14 or later. A no-collection privacy
manifest is provided at `mobile/ios/PrivacyInfo.xcprivacy`.

`scripts/prepare_ios.sh` is deliberately gated by
`GWT_CONFIRM_IOS_BUILD=yes`. Per project policy, the script has not been run and
the archive has not been integrated into a Godot-exported Xcode project yet.
That build, static-archive merge/XCFramework integration, code signing, device
execution, and App Store validation remain in the explicitly deferred iOS
build/test task.

References:

- [Android NDK downloads](https://developer.android.com/ndk/downloads)
- [Apple Xcode 15 release notes](https://developer.apple.com/documentation/xcode-release-notes/xcode-15-release-notes)
- [Apple privacy manifest guidance](https://developer.apple.com/documentation/bundleresources/adding-a-privacy-manifest-to-your-app-or-third-party-sdk)

## Device handoff matrix

Before a production mobile release, run these device checks. Packet capture is
not required; use safe trace events and server session logs.

| Scenario | Expected result |
| --- | --- |
| Background then foreground | Old session closes; resume signal fires; application reconnects |
| Wi-Fi to cellular | Network monitor calls `handle_network_change()`; old session closes; reconnect succeeds |
| Cellular to Wi-Fi | Same deterministic close/reconnect policy; no crash |
| IPv6-only NAT64 | DNS hostname connection succeeds without an IPv4 literal |
| Offline then online | Initial failure is retryable; application reconnects after path availability |

Physical Wi-Fi/cellular and App Store tests cannot be automated by desktop CI.
Record device model, OS version, network transition, close signal, reconnect
latency, and server-observed close reason for each release candidate.
Current host and Android ABI results are recorded in
[mobile-validation.md](mobile-validation.md).
