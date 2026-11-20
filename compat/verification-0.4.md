# KromaRec 0.4 verification

All KromaRec tests ran on an x86-64 Debian 13 host.

| Milestone | Result |
| --- | --- |
| A and B authenticate | Pass with independent HTTP clients using local password grants |
| A creates a room | Pass through the legacy template clone route and the Studio create route |
| B discovers and enters it | Pass through public search and 2023 form-encoded matchmake. Unpublished rooms reject unrelated players |
| Both share one instance | Pass. Concurrent requests returned the same numeric instance ID, scene and Photon room name. A fifth player was assigned an overflow instance when a four-player room filled |
| Multiplayer transport connection | Photon connection-info contract and signed credentials validated over HTTP. A separate Luxon 1.5 process successfully bound the UDP Name, Master and Game ports. No Photon application-level handshake was exercised |
| Shared gameplay | Unverified. Requires two patched 20230414 game clients to connect, load the scene and replicate interactions |

The regression suite passed with both static musl and glibc release builds.
The room suite additionally covers owner-only deletion, cascading cleanup,
ownership transfer, durable co-owner invitations, private instance joins,
recipient-only room invitations, role permissions, heartbeat/logout, stale
location expiry, persistence across restart, migration of an existing
0.3 account plus settings, SQLite foreign keys and integrity.
The SignalR suite performs a real WebSocket upgrade and JSON handshake,
round-trips a ping and reads targeted room, invite and friend-presence frames.
A friend never receives the Photon room name through these notifications.

## RSS and binary size

The musl comparison below uses the median of three fresh runs per executable.
Each run used the same regression workload, including 100 simultaneously open
HTTP keep-alives for the warm snapshot. Values are process RSS in KiB.

| Build | Binary bytes | Idle RSS | Warm RSS |
| --- | ---: | ---: | ---: |
| 0.3 static musl | 2,923,904 | 2,840 | 5,456 |
| 0.4 static musl | 3,313,056 | 3,388 | 6,016 |
| Change | +389,152 | +548 | +560 |
| 0.4 glibc, single run | 3,187,496 | 4,056 | 25,536 |

In a separate room-suite run, static musl reached 3,492 KiB RSS after two
HTTP players entered and one SignalR socket opened. Luxon 1.5 was measured
separately at 8,560 KiB RSS immediately after startup with no clients.
These snapshots do not measure peak Argon2, TLS, transport load or gameplay.
Alpine userspace, OpenRC execution, the real Rec Room client and a live
differential run against a deployed RecFlare server were not exercised.

## Reproduce

```sh
cargo fmt --check
cargo build --locked --release
python3 tools/smoke.py target/release/kromarec
python3 tools/rooms_smoke.py target/release/kromarec --baseline /path/to/kromarec-0.3
# To check the packaged musl binary, pass bin/kromarec to both Python scripts.
```
