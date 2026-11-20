# KromaRec 0.9.0 verification

Base revision `c70493d` (0.8.0). The compatibility inventory retains the
same 689 rows from RecFlare 0.0.10, pinned at
`86bdd02063629677f120916d5d419440e0aeee71`. No new Cargo dependencies were
required. The integration tests used the 0.8 bundle's existing patched
Luxon 1.5 executable.

| Test | Result |
| --- | --- |
| Rust release builds | x86_64 GNU and x86_64 static PIE musl passed with Rust 1.100.0 on Debian 13. The musl ELF has no dynamic library dependencies. |
| Rust regression | Both release targets passed the cancellation regression for serialized SQLite access. |
| HTTP and SQLite | Both builds passed bootstrap, local account signup/login, concurrent reuse of a refresh token (one accepted), expired or malformed bearer rejection, password changes, Orientation/dorm DTOs, room creation/discovery, race-free allocation, capacity, invitations, presence and SignalR. The migration test used a database created by the 0.8 executable and confirmed settings, accounts, rooms, valid access and unused refresh tokens, schema version 7, WAL, foreign keys, integrity and restart persistence. |
| Photon Realtime | Both builds passed the suite against patched Luxon 1.5 on loopback UDP. The probe fetched regions from NameServer, authenticated, walked Master and Game and used separate actors in one Photon room. Two peers exchanged bidirectional events, left and rejoined, then disconnected at the transport layer and rejoined. |
| Photon authorization | Luxon rejected missing, corrupted, expired, old-format or wrong-purpose and wrong-audience tickets. The signed subject replaced the claimed user ID. The suite covered a ban taking effect before Master auth, a password change before Game auth, and logout after Game auth but before joining. Knowing a room name did not bypass a reservation. A fresh ticket let authorized users rejoin. |
| Alpine Linux 3.24.4 userspace | The static binary ran inside an Alpine root filesystem, served 20230414 version check, created an account in SQLite and exited cleanly on SIGTERM. |
| OpenRC | Verified the scripts are installable and OpenRC 0.63.2 registered the service. The namespaced OpenRC startup attempt stopped at Alpine's hostname dependency with `Operation not permitted`. Full startup/shutdown through OpenRC on a booted Alpine system remains pending. |
| Native Rec Room 20230414 | Unverified. No Windows game installation or generated IL2CPP assemblies were available. |

## Measured RSS on Debian 13

Idle and warm values are medians of three runs. The warm sample follows
the same HTTP workload and includes 100 open HTTP keep-alives. The two-peer
sample was taken during a successful Photon room session on each build.

| Build | Executable bytes | Idle KromaRec | Warm KromaRec | KromaRec with two peers | Luxon with two peers | Combined |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0.8 glibc baseline | 3,268,488 | 4,116 KiB | 25,696 KiB | — | — | — |
| 0.9 static musl | 3,415,456 | 3,320 KiB | 6,052 KiB | 3,600 KiB | 3,756 KiB | 7,356 KiB |
| 0.9 glibc | 3,279,560 | 4,076 KiB | 25,696 KiB | 23,936 KiB | 3,764 KiB | 27,700 KiB |

RSS snapshots are process measurements and exclude clients, a TLS proxy,
and peak memory. The combined column sums both processes' RSS. The 0.9
musl idle/warm pairs in KiB were (3284, 6100), (3360, 6052), and
(3320, 6016). The glibc pairs were (4072, 25692), (4076, 25740), and
(4136, 25696).

## Reproduce

From the archive root, with Python 3 and a Linux host:

```sh
cargo fmt --check
cargo test --locked --release
cargo test --locked --release --target x86_64-unknown-linux-musl
python3 tools/smoke.py bin/kromarec
python3 tools/smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec --baseline /path/to/kromarec-0.8 \
  --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec-x86_64-linux-gnu --baseline /path/to/kromarec-0.8 \
  --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/memory_rss.py bin/kromarec bin/kromarec-x86_64-linux-gnu
python3 tools/memory_rss.py --schema 6 /path/to/kromarec-0.8
sha256sum -c SHA256SUMS
```

The Rust musl test command requires the musl target plus musl C headers and
a linker, or a native Alpine toolchain. See the [README](../README.md).

## Remaining client milestones

Start with the patched Windows client loading successfully, discovering RecNet
over trusted HTTPS, signing in and passing a real ticket to the encrypted
Photon NameServer call. Then verify Orientation and dorm rendering, followed
by visible movement and interactions in both directions and a leave/rejoin
with two clients. The protocol probe used plaintext on loopback and does not
exercise Unity networking, game RPCs, motion or scene rendering.

Platform authentication, Photon Voice and device ID migration remain pending.
The deployment guide covers the existing startup workaround and required game
assets. Password changes and normal logout reject old Photon tickets at the
next auth or admission check. They do not force active actors or existing
SignalR sockets off the server.

[Two-client deployment and acceptance](multiplayer.md)
