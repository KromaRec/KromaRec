# KromaRec 0.10.1 verification

**Native Rec Room PC 20230414 startup and gameplay: UNVERIFIED.** All network clients used in this report were Python probes. They connected to the release executables, SQLite, Caddy and patched Luxon on Linux.

## Implemented and exercised

| Area | Actual result |
| --- | --- |
| Release build | Built optimized x86-64 glibc and static PIE musl KromaRec with Rust 1.100.0 on Debian 13. The musl ELF has no interpreter or NEEDED library entries. Built and stripped Luxon 1.5 from commit e4bab98d8d6b3aee9ecf4035309a78a59b584f93 with the shipped patches and Mbed TLS 3.6.5. The bundled Luxon uses glibc. |
| SQLite and concurrency | Both targets passed both Rust tests, covering cancellation while blocking work holds the SQLite permit and overlapping revoked, unrelated and reconnected leases. The room suite passed concurrent allocation, private admission, capacity, repeated party fields and invitation delivery. The musl suite opened a database made by the 0.9 executable and migrated schema 7 to 8. The final glibc suite reopened an actual 0.10.0 schema 8 database. Account credentials, settings, reservations, room coordinates and restart persistence passed. |
| HTTP and TLS | Both targets passed local signup, password auth, concurrent refresh, revocation, startup and CDN tests. Behind Caddy 2.6.2 the untrusted CA was rejected; the trusted preflight, Orientation metadata, signup, negotiation and authenticated SignalR WSS handshake/ping passed. SIGTERM exited successfully. |
| SignalR | Active sockets closed on password change, normal logout and expiration of a short-lived HTTP bearer. Existing reconnection, presence and invite tests passed. Admission registers the lease under the serialized DB operation. Lease invalidation runs after commit inside that operation. A normal logout still allows a valid HTTP bearer to open a new socket. Photon-only bans leave HTTP and SignalR access policy intact. |
| Photon | Name, Master and Game passed valid handoffs and the existing suite of invalid/expired/revoked ticket, identity, ban and reservation tests. Two admitted synthetic peers exchanged bidirectional events with distinct sender identities, left and rejoined. Both targets passed party/private instance checks. |
| Live Game eviction | The victim received a server-initiated ENet Disconnect after password change, normal logout and an operator SQL ban. Its observer received a Leave event identifying the correct actor and stayed connected. A fresh login rejoined. Healthy peers exchanged 15 round trips across the lease watchdog interval. Room deletion, suspending KromaRec with SIGSTOP, and terminating KromaRec each disconnected an active Game peer. |
| Diagnostics | Opt-in HTTP status labels and recognized scene IDs passed. The suite observed distinct Name, Master and Game auth labels plus accepted and rejected Game admission. The captured logs passed checks for absence of test credentials, arbitrary request data, tickets and gate key. |
| Alpine and OpenRC | Ran the static executable in an Alpine 3.24.4 root filesystem, read Orientation metadata, created an account in SQLite and exited on SIGTERM. Booted OpenRC startup/shutdown and a native Alpine build of the new Luxon were not exercised. |
| Inventory | Compared the complete route-status TSV against the 0.10.0 package. All 689 data rows and their classifications are byte-identical. |

## Timing and failure behavior

During parallel testing an intermediate build returned Photon 32753 on a Name-to-Master handoff. Luxon kept the handoff state in the old handler until deferred destruction, and the probe slept 100 ms before proceeding. The release publishes that state during disconnect and the harness waits for the ENet acknowledgement. Both final suites passed, including repeated handoffs and reconnects. Packet loss, WAN timing and long-duration soak behavior remain unmeasured.

Password changes and completed logout signal the account's active leases after commit. SignalR cancels its socket task. Luxon observes gate closure on its next slow update and takes its normal disconnect path. This is not a synchronous network barrier at the instant of the HTTP response. Out-of-process SQL bans, deleted rooms and expired reservations are checked every five seconds per admitted actor. Each database recheck has a two-second timeout. A healthy check renews the lease. A stalled backend stops renewals and the Luxon slow update enforces a 12-second watchdog. There is no per-event SQLite check or idle polling worker in KromaRec. Ticket expiry governs new admissions and handoffs; a healthy, continuously reserved actor uses its live lease.

## Resource measurements

HTTP figures are medians of five process runs. RSS is from Linux smaps_rollup at idle and with 100 open HTTP keep-alives. CPU is process user plus system time from procfs. The idle sample interval was two seconds.

| Binary | Bytes | Idle RSS (KiB) | Warm RSS (KiB) | Idle CPU (s) | HTTP CPU (s) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0.10.0 musl, published | 3,427,744 | 3,452 | 6,144 | 0.000 | 0.350 |
| 0.10.0 musl, rerun here | 3,427,744 | 3,380 | 6,072 | 0.000 | 0.480 |
| 0.10.1 musl | 3,456,416 | 3,448 | 6,132 | 0.000 | 0.440 |
| 0.10.1 glibc | 3,323,880 | 4,268 | 25,724 | 0.000 | 0.380 |

The musl file grew 28,672 bytes. Its same-host idle/warm medians rose by 68/60 KiB; both releases' idle and warm samples overlap. HTTP CPU varied between measurement sessions. Zero means the median idle interval accumulated no full CPU tick.

| Two admitted peers, RSS snapshot | KromaRec KiB | Luxon KiB | Combined KiB |
| --- | ---: | ---: | ---: |
| 0.10.0 musl, published | 3,636 | 3,748 | 7,384 |
| 0.10.0 musl, same-host rerun with schema 7 baseline | 3,716 | 3,804 | 7,520 |
| 0.10.1 musl | 3,636 | 3,924 | 7,560 |
| 0.10.1 glibc | 23,772 | 3,936 | 27,708 |

The final musl pair used 176 KiB more than the published snapshot and 40 KiB more than the same-host rerun. Each active actor now holds a bounded loopback socket and account lease. RSS omits kernel socket memory. One earlier glibc run reached 43,780 KiB at the later open-SignalR stage; the final glibc run read 24,084 KiB at that stage. The reason for that variation was not isolated. These readings are snapshots, not peak memory or limits.

Over 15 seconds with one bidirectional application exchange per second, the final musl pair accumulated 0.030 s backend CPU and 0.050 s Luxon CPU. The glibc pair accumulated 0.030 s and 0.040 s respectively. There is no equivalent published 0.10.0 active CPU workload.

## Native blockers in acceptance order

1. Obtain two genuine Windows 20230414 installations, generated IL2CPP interop and the version-matched KromaRec-Patch 0.3.x build. The available public main checkout identified itself as 0.2.0. The 0.3.x artifact, hook bindings and original security checks were not locally verified.
2. Verify trusted HTTPS discovery, local manual login, device handling and authenticated startup with those original checks active. The device-ID route remains pending. Record the first failed route/status or client hook without credentials or device identifiers.
3. Render Orientation and each player's dorm. Server metadata and a CDN preflight do not establish scene rendering or asset completeness.
4. Capture native encrypted Name, Master and Game authentication, parameter-216 ticket delivery, matching room/instance/scene and two distinct actors. The wire probe uses loopback plaintext.
5. Observe spawning, visible movement and interactions from both Windows clients before and after leave/rejoin. Generic event relay establishes no Unity serialization or gameplay result. Voice also remains unverified.

## Reproduction

From the extracted project, with Caddy and Python 3 available:

```sh
cargo fmt --check
cargo test --release --locked
cargo test --release --locked --target x86_64-unknown-linux-musl
python3 tools/smoke.py bin/kromarec
python3 tools/smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec --baseline /path/to/kromarec-0.9 --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec-x86_64-linux-gnu --baseline /path/to/kromarec-0.10.0 --baseline-schema 8 --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/https_proxy_smoke.py bin/kromarec
python3 tools/https_proxy_smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/memory_rss.py --runs 5 /path/to/kromarec-0.10.0 bin/kromarec bin/kromarec-x86_64-linux-gnu
sha256sum -c SHA256SUMS
```

See [deployment and native acceptance](multiplayer.md) for port assignments, certificate trust, OpenRC and the two-client observation checklist.
