# Mobile Validation Record

Date: 2026-07-11

## Automated and local results

| Check | Result | Evidence |
| --- | --- | --- |
| Android arm64 debug build | passed | NDK r29, API 24, AArch64 ELF shared object |
| Android arm64 release build | passed | AArch64 ELF; insecure TLS symbol absent |
| Pause with an active session | passed | Session closed; lifecycle signal count was one |
| Resume notification | passed | Resume signal fired; a new connection succeeded |
| Network-change handoff | passed | Active session closed; reconnect-required signal fired |
| Concurrent sessions | passed | Two sessions echoed independently with distinct stable IDs |
| IPv6-only socket path | passed | Godot TLS/datagram round trip to `[::1]` |
| iOS build and runtime | deferred | Explicitly excluded until requested by the project owner |

The pause/resume and network-change checks invoke the same Godot notifications
and public method used by mobile integrations, on the macOS test host. The
Android artifact is compile- and ABI-verified; an emulator or physical device
runtime is still required before publishing an Android release.

## Required release-candidate device checks

Physical Wi-Fi/cellular switching, Android process suspension, Doze behavior,
and NAT64 depend on carrier and device state and cannot be proven by the host
tests. Execute the matrix in `docs/mobile.md` on at least one target Android
device. iOS remains outside the current execution scope.
