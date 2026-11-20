# KromaRec 0.10.0 verification

The base is the delivered 0.9.0 source and binaries. The 689 inventory rows
still match the pinned RecFlare 0.0.10 source identities. The release carries
schema 8 and has no new Cargo dependencies. The package includes the
existing patched Luxon 1.5 binary and its reproducible patch/build inputs.

## Results

| Check | Result |
| --- | --- |
| Linux release builds | x86_64 glibc and static PIE x86_64 musl passed with Rust 1.100.0 on Debian 13. The musl ELF has no dynamic interpreter or shared library dependencies. |
| Rust, HTTP and auth regression | Both targets passed the Rust SQLite-cancellation test and the HTTP smoke suite. Coverage includes local signup, login, concurrent refresh, password changes, JWT rejection, startup, settings, friends and CDN byte ranges. |
| Schema 7 → 8 and persistence | An actual 0.9 binary generated the database, account credentials, settings, room and a live reservation. After migration both targets returned the same room/instance/Photon coordinates and retained the old access and refresh credentials. Default policy, SQLite WAL, integrity, foreign keys and restart persistence were checked. |
| HTTPS and WSS | Both binaries passed the shipped Caddy host routing smoke test on Caddy 2.6.2. The client rejected the untrusted CA, passed the trusted preflight, created a local account, negotiated SignalR and authenticated a WSS connection. The preflight checked the Orientation Unity scene ID. |
| Party and room allocation | Both builds accepted repeated `AdditionalPlayerIds`, deduplicated duplicate recipients, delivered legacy invitations over SignalR, refused a direct Photon join without a reservation, and let invited accounts allocate into the same private instance. The suite also checked room capacity, Default/Avoid/Ignore, member-only policy changes and owner-only closure to public allocation. |
| Photon Realtime and security | Two synthetic peers used patched Luxon 1.5 on loopback UDP through NameServer, Master and Game, exchanged events with correct sender actor IDs, and left, disconnected and rejoined. Coverage retained invalid/expired tickets, bans, password revocation, logout and unauthorized reservations at the existing transport gates. |
| Alpine runtime | The static binary in Alpine 3.24.4 userspace returned Orientation metadata, registered a local account in SQLite and exited cleanly on SIGTERM. A booted Alpine/OpenRC environment was unavailable for end-to-end service lifecycle testing. |
| Native 20230414 | No Windows installation, generated IL2CPP assemblies or game traces were available. Native startup, scene load and gameplay remain unverified. |

## Measurements

Debian 13, Linux process RSS snapshots. The idle and warm columns are
medians of five runs. Warm measurements follow the HTTP/auth regression
workload with 100 open HTTP keep-alive connections. The CPU column is the
median user plus system CPU time of the backend across the workload,
measured with `/proc/PID/stat`. The median two-second idle window recorded zero full CPU ticks in each
series. The two-peer measurements are separate
snapshots and exclude clients and the optional TLS reverse proxy.

| Build | Bytes | Idle RSS | Warm RSS | HTTP CPU |
| --- | ---: | ---: | ---: | ---: |
| 0.9 musl baseline | 3,415,456 | 3,276 KiB | 6,072 KiB | 0.340 s |
| 0.10 static musl | 3,427,744 | 3,452 KiB | 6,144 KiB | 0.350 s |
| 0.10 glibc | 3,288,648 | 4,160 KiB | 25,680 KiB | 0.260 s |

The musl series is 176 KiB higher idle and 72 KiB higher warm than the
same-host baseline. Its idle samples overlap the baseline. An initial
three-run glibc series included one 45,224 KiB warm sample. All five
warm reruns fell between 25,668 and 25,704 KiB. The cause of the outlier
remains unverified. These are observations rather than process limits.

| Two-peer protocol session | KromaRec | Luxon | Combined |
| --- | ---: | ---: | ---: |
| 0.10 musl | 3,636 KiB | 3,748 KiB | 7,384 KiB |
| 0.10 glibc | 23,876 KiB | 3,760 KiB | 27,636 KiB |

## Reference and remaining blockers

1. **Startup and local authentication.** Confirm the 20230414 IL2CPP patch
   loads, trusts the deployment CA and reaches manual sign-in on Windows.
   The bootstrap catalog matches the pinned 2023 catalog except the
   existing telemetry disablements. The pinned
   [device report handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/api/src/routes/moderation.ts#L1054)
   documents a native account-creation hang. KromaRec leaves device-ID
   submission pending and retains the documented client workaround.
2. **Scene loading and encrypted handoff.** Run the checklist through
   Orientation and dorm rendering and confirm the native client supplies
   the signed ticket to Photon. Compare the returned region, Photon room
   and Unity scene identifiers on both clients. If scene loading fails,
   record the scene/save ID and the first missing CDN request with secrets
   removed. Synthetic HTTPS and generic Photon UDP operations do not
   exercise Unity's serialized traffic or Photon session encryption.
3. **Gameplay replication.** Observe spawning, avatar state, movement,
   rotation, gestures and interactions in both directions in the same
   instance. Repeat after leaving, reconnecting and following a party
   invite. The generic event probe asserts relay and actor identity. No
   undocumented Unity event formats were added.
4. **Active-session invalidation and remaining features.** The existing
   Luxon hook checks authentication and admission. A connected actor can
   still relay until leaving or being disconnected by the transport.
   Immediate forced eviction and immediate closure of connected SignalR
   sockets were not added. Photon Voice, game room saves and custom
   subroom selection remain outside this verification.

Party invitation handling follows the repeated form fields and legacy
message shape in the pinned
[match worker](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/match/src/match.app.ts#L1495).
Party fan-out is capped at 16 distinct targets. Invitations bind to the
allocated instance. An invite on its own does not grant Photon access.

## Reproduction

```sh
cargo fmt --check
cargo test --release --locked
cargo test --release --locked --target x86_64-unknown-linux-musl
python3 tools/smoke.py bin/kromarec
python3 tools/smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec --baseline /path/to/kromarec-0.9 \
  --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec-x86_64-linux-gnu --baseline /path/to/kromarec-0.9 \
  --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/https_proxy_smoke.py bin/kromarec
python3 tools/https_proxy_smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/memory_rss.py --runs 5 --schema 7 /path/to/kromarec-0.9
python3 tools/memory_rss.py --runs 5 bin/kromarec bin/kromarec-x86_64-linux-gnu
sha256sum -c SHA256SUMS
```

See [deployment and native acceptance](multiplayer.md) for the two-client
checklist and OpenRC steps.
