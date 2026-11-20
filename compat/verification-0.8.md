# KromaRec 0.8.0 verification

| Evidence | Coverage |
| --- | --- |
| Verified with native client | No run. |
| Verified with synthetic integration tests | Optimized musl and glibc builds passed bootstrap, local authentication, refresh rotation, account/settings/room persistence from 0.7, concurrency, stale logout, room capacity, private invitations, and the patched Luxon Name/Master/Game tests with Authenticate, AuthenticateOnce, JoinGame, rejects and leave/rejoin. SignalR reconnect handles a handshake and ping in one WebSocket message, then delivers room and invitation events. Startup diagnostics were checked with query, body, path and HTTP-method markers. |
| Verified with synthetic HTTPS | The client-side preflight passed through a local TLS test proxy with a test CA and rejected an untrusted certificate and mismatched MatchUrl. This did not use Caddy or the game TLS implementation. |
| Implemented but unverified | The Phase 1 source patch still needs an IL2CPP build and in-game session. A native startup trace is required to establish the actual request order. |
| Not implemented | Device ID migration, platform-linked login, and Photon Voice. The pinned device ID route accepts form oldDeviceId, newDeviceId and platform. Upstream documents that its current empty-array response hangs during account creation. |
| Blocked by missing external dependencies | Original Windows 20230414 depot and generated IL2CPP assemblies. The first native gate is plugin load and trusted HTTPS discovery, then manual login and an encrypted NameServer connection. Scene rendering, game replication and reconnect remain unverified. |

Both builds used Rust 1.100.0 with the locked dependencies. No new runtime
dependencies or SQLite migration were required. Fresh OpenRC and Alpine
runtime lifecycle tests were not run in this phase.

## RSS on Debian

Idle and warm medians from three runs of the smoke workload. Warm means 100
open HTTP keep-alives. Paired samples come from the secure two-peer test.

| Build | Binary bytes | Idle KiB | Warm KiB | Backend with two peers | Luxon | Combined |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0.7 static musl baseline | 3,394,976 | 3,364 | 6,068 | — | — | — |
| 0.8 static musl | 3,403,168 | 3,344 | 6,072 | 3,620 KiB | 3,660 KiB | 7,280 KiB |
| 0.8 glibc | 3,268,488 | 4,148 | 25,684 | 23,756 KiB | 3,756 KiB | 27,512 KiB |

The static build changed by −20 KiB idle, +4 KiB warm and +8,192 bytes on
disk compared with the restored 0.7 baseline. Snapshot RSS excludes
clients, TLS proxy, and peak memory. Repeat with `tools/memory_rss.py` and
`tools/rooms_smoke.py --secure-luxon ...`.

See [deployment and two-client acceptance](multiplayer.md).
