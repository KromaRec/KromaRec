# KromaRec 0.10.4 verification

**Native Rec Room 20230414 multiplayer compatibility remains unverified.** The Windows patch handoff records the native discovery certificate verifier rejecting the request before an HTTP response. This release reproduces a separate backend Orientation failure and fixes its configuration handling.

## Provenance and scope

The source baseline was the verified 0.10.3 Linux distribution ZIP, SHA-256 `c0bbca1e6c2f6d63fba75f2185ec265f86878ce0a1dae1716e3cc07e9f936dc2`. It contains source without upstream Git metadata. The archive digest identifies the baseline. The pinned RecFlare checkout was `86bdd02063629677f120916d5d419440e0aeee71`. The tests used Debian 13 x86-64, ephemeral ports, newly initialized databases and isolated Caddy trust. No production data or deployment was accessed.

## Corrections

| Request or interface | Reproduced on 0.10.3 | 0.10.4 result |
|---|---|---|
| rooms GET /rooms/13 with a configured DB path and absent signing secret | Empty 503, even as ns GET / returned the directory. The process silently ignored the DB path. | Startup rejects this configuration before opening the DB. With the existing secret loaded, Orientation returns 200. The startup banner distinguishes persistent and stateless mode. |
| match PUT /player/photonregionpings and /player/gameserverregionpings | Empty 401 for anonymous requests at the generic session dispatcher. | Exact method and paths return the pinned empty 200 acknowledgement. They have no state or identity effect. Reservation admission still requires authorization. |
| accounts GET and POST /account/bulk | Both returned 404 and the JSON fallback. | Public profiles from SQLite in first-occurrence order. Query IDs and URL-encoded form IDs accept repeated keys and comma-separated lists. Unknown IDs use the reference's public default. The 100-ID ceiling, stricter integer parser and supported POST media type are documented limits. |
| Shared URL-encoded form parsing | Case-sensitive prefix comparison rejected valid media-type casing and accepted unrelated types with the same prefix. | Exact case-insensitive media-type match, optional parameters. Regression tests exercise a mixed-case authorized format and reject a lookalike with 415. |
| Matrix refresh | The historical generator rebuilt rows from an older list, discarding maintained partial-implementation decisions. | Refresh preserves keyed decisions, rejects ambiguous or removed rows for review and keeps the pinned 689 entries. |
| Version inspection | No dedicated version-only invocation. | Both executables print KromaRec 0.10.4 with --version and exit before opening storage. |

The exact fallback was `{"success":false,"error":{"message":"not found"}}`. The reported literal success/error string was not reproduced. Full root bodies, selected path probes and exact anonymous route errors are in the [host matrix](../compat/service-hosts-0.10.4.md), [browser diagnostics](../compat/api-errors-0.10.4.md), [route table](../compat/routes-0.10.4.md) and [route JSON](../compat/routes-0.10.4.json). The historical operator's 503 could have another cause. Do not recreate a production database to troubleshoot it.

## Test results

| Status | Workload | Evidence |
|---|---|---|
| PASS | Builds | Rust 1.100.0, release profile and locked dependencies. Static PIE x86-64 musl, 3,554,720 bytes. Dynamic x86-64 glibc, 3,422,440 bytes. Rust formatting and two unit tests per target passed. |
| PASS | Distribution | ZIP CRC, clean extraction, internal SHA-256 manifest and executable permissions verified. Both extracted backends passed HTTP smoke. The extracted musl binary also passed Caddy HTTPS/WSS, all service hosts and the inventory sweep. |
| PASS | HTTP, auth and error paths | Both compiled binaries exercised account creation, login, refresh, password changes, invalid credentials, settings, social and bootstrap APIs. New bulk, ping acknowledgement, media-type and missing-secret checks ran inside the existing smoke suite. |
| PASS | All 33 origins and ns | Trusted test CA, SNI, preserved Host, exact root comparison, missing routes, alias collisions, version checks, loading tips, Orientation, signup and authenticated WSS. |
| PASS | Inventory reconciliation | 689 pinned source locations checked. 98 implemented, 37 partial, 536 missing, 4 telemetry exclusions and 14 router mounts. 671 anonymous method/path probes received 52 HTTP 200, 3 HTTP 400, 118 HTTP 401, 1 HTTP 403, 496 HTTP 404 and 1 HTTP 415. No 5xx. Twenty-eight rows also passed explicit sample assertions in this sweep. Other suites test additional authenticated cases. Exhaustive contract parity remains untested. |
| PASS | SQLite and restart | Fresh schema 9, populated 0.10.3 schema 9 reopen, 0.10.1 schema 8 migration to 9, retained account/login, refresh state, settings and room. WAL online backup reopened with Orientation accessible, integrity_check ok, foreign_key_check empty. No migration or application dependency added. |
| PASS | SignalR and sessions | Negotiation, JSON handshake, ping, authenticated WSS, reconnect, invitations, active credential invalidation and room privacy. Scoped cancellation, concurrent matchmaking, generation checks and shutdown exercised by the existing suites. |
| PASS | Photon/Luxon on loopback | Exercised the 0.10.3 bundled patched **glibc** Luxon 1.5. Name, Master and Game admission, bad and expired tickets, bans, private reservations, live evictions, disconnect/rejoin, room deletion and stalled-backend shutdown. Public gate bind rejected. |
| PASS | Encrypted synthetic authentication | Separate KromaRec-Patch probe using fresh backend accounts counted 36 encrypted successful NameServer Authenticate operations and one AuthenticateOnce. Master and Game handoffs completed. Encryption of every subsequent game operation is outside this result. |
| PASS | Orientation and dorm | Synthetic account entered Orientation room 13/subroom 23. Independent dorm allocation returned a private dorm location. In each case metadata and the assigned Photon room agreed, and the Game server admitted the actor. |
| PASS | Multiplayer event transport | Ten synthetic actors in one RecCenter verified all 900 deliveries. Twenty actors in one Park instance verified all 3,800. Tests asserted sender and payload before leaving. |
| FAIL | Last documented Windows native discovery attempt, 0.10.3 | Request-specific TLS verifier returned false and TlsFatalAlert. Earlier independent Python TLS preflight succeeded using the Caddy CA. |
| BLOCKED | Current native Windows startup and two-player gameplay | No authorized Windows game runtime in the test environment. No native scene rendering, movement, interaction, login or reconnection evidence for 0.10.4. |
| NOT TESTED | External deployment | Booted Alpine/OpenRC, Tailscale/LAN DNS, packet loss, Android/Quest, platform auth, voice and long-running soak tests. |

The full host table below records each service root and a representative non-root request when available. The resolver lacked the operator's private DNS. The test routes HTTPS connections to the local proxy with certificate verification active.

| Host | HTTPS GET / | Probe |
|---|---:|---|
| ai.rec.test | 200 | GET /gameai/user/access → 401 |
| api.rec.test | 404 | GET /api/versioncheck/v4?v=20230414 → 200 |
| accounts.rec.test | 200 | GET /account/me → 401 |
| auth.rec.test | 404 | POST /connect/token → 415 |
| cdn.rec.test | 200 | GET /config/LoadingScreenTipData → 200 |
| cms.rec.test | 404 | No non-root route in pinned inventory |
| cards.rec.test | 200 | GET /iam/me/channels/Announcements → 401 |
| chat.rec.test | 200 | GET /thread/1/message → 404 |
| clubs.rec.test | 404 | GET /club/1 → 404 |
| commerce.rec.test | 200 | GET /purchase/v1/hasspentmoney → 200 |
| data.rec.test | 404 | No non-root route in pinned inventory |
| discovery.rec.test | 200 | GET /sections/bulk → 200 |
| econ.rec.test | 404 | GET /api/influencerpartnerprogram/influencers → 404 |
| geo.rec.test | 404 | No non-root route in pinned inventory |
| img.rec.test | 200 | GET /openapi.json → 404 |
| leaderboard.rec.test | 200 | GET /openapi.json → 404 |
| link.rec.test | 200 | POST /actionlink → 404 |
| lists.rec.test | 200 | GET /curatedlists → 200 |
| match.rec.test | 401 | GET /player/connection-info → 401 |
| moderation.rec.test | 200 | GET /voice/config → 200 |
| notify.rec.test | 404 | POST /hub/v1/negotiate?negotiateVersion=1 → 200 |
| platformnotifications.rec.test | 200 | GET /preferences → 404 |
| playersettings.rec.test | 200 | GET /playersettings → 401 |
| roomcomments.rec.test | 200 | GET /comments/get/1 → 200 |
| roomieintegrations.rec.test | 404 | No non-root route in pinned inventory |
| rooms.rec.test | 200 | GET /rooms/13 → 200 |
| storage.rec.test | 200 | GET /openapi.json → 404 |
| strings.rec.test | 404 | No non-root route in pinned inventory |
| strings-cdn.rec.test | 404 | No non-root route in pinned inventory |
| studio.rec.test | 200 | GET /cloud-builds → 400 |
| thorn.rec.test | 404 | No non-root route in pinned inventory |
| videos.rec.test | 404 | No non-root route in pinned inventory |
| www.rec.test | 404 | GET /privacy → 404 |
| ns.rec.test | 200 | GET /, 33 distinct HTTPS origins |

## Memory, CPU and latency

Linux process RSS snapshots in KiB. Backend measurements use the static binary. The HTTP benchmark holds 100 keep-alive connections and reports medians of three runs. Photon results use the process counters and ten timed rounds with one-second pauses. Prior numbers come from the 0.10.3 release report.

| Workload | 0.10.3 | 0.10.4 |
|---|---:|---:|
| Backend idle | 3,804 | 3,804 |
| Backend warm HTTP | 6,396 | 6,376 |
| Idle CPU over 2 s / HTTP workload CPU, seconds | 0.000 / 0.450 | 0.000 / 0.430 |
| Two peers RSS, backend + Luxon | 3,904 + 3,892 = 7,796 | 3,908 + 3,784 = 7,692 |
| Ten peers RSS, backend + Luxon | 3,928 + 5,368 = 9,296 | 3,952 + 5,044 = 8,996 |
| Twenty peers RSS, backend + Luxon | NOT TESTED | 3,976 + 6,840 = 10,816 |
| Two peers CPU over 15 s, backend / Luxon | 0.000 / 0.010 | 0.010 / 0.030 |
| Ten peers CPU, backend / Luxon and elapsed seconds | 0.000 / 0.020 in 10.654 | 0.000 / 0.030 in 10.410 |
| Twenty peers CPU, backend / Luxon and elapsed seconds | NOT TESTED | 0.010 / 0.050 in 10.828 |

In the separate encrypted NameServer run the two-peer snapshot was 7,776 KiB, and ten peers used 9,224 KiB. These short samples measure neither peak memory nor a sustained performance improvement. A zero CPU delta means no measurable scheduler tick.

A separate same-host A/B run opened 100 fresh HTTP connections for each path. The fresh database startup time was 5.099 ms on 0.10.3 and 4.934 ms on 0.10.4. Discovery p50/p95 was 0.208/0.273 ms and 0.210/0.261 ms respectively. SQLite-backed Orientation p50/p95 was 0.302/0.409 ms and 0.292/0.394 ms. These samples include loopback HTTP overhead.

## Runtime audit boundaries

The DB uses one connection, an async permit carried into blocking work, immediate migration transactions, a 2 s busy timeout, foreign keys and WAL. Unit tests cover cancellation while the DB job is still running and stale leases after reconnect. SignalR bounds active sockets at 128, uses a 16-message per-account broadcast channel and applies timeouts to the handshake and writes. Registry locks are synchronous and the relevant guards end before socket awaits. Existing invariant and mutex-poison checks remain. This audit provides no general proof of absence of deadlocks, leaks or panics.

## Reproduce from the extracted release

Use a clean shell with Python 3.11+, Caddy and a compatible glibc runtime for Luxon. Tests start their own disposable services. For compiling, install the Rust targets and a musl C linker for bundled SQLite.

```sh
sha256sum -c SHA256SUMS
bin/kromarec --version
cargo fmt --check
cargo test --locked --release --target x86_64-unknown-linux-gnu
cargo test --locked --release --target x86_64-unknown-linux-musl
cargo build --locked --release --target x86_64-unknown-linux-gnu
cargo build --locked --release --target x86_64-unknown-linux-musl
python3 tools/smoke.py bin/kromarec
python3 tools/smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/service_hosts_test.py bin/kromarec --caddy caddy
python3 tools/https_proxy_smoke.py bin/kromarec --caddy caddy
python3 tools/route_audit.py bin/kromarec --output /tmp/kromarec-audit
python3 tools/startup_probe.py bin/kromarec --baseline /path/to/0.10.3/bin/kromarec
python3 tools/rooms_smoke.py bin/kromarec --baseline /path/to/0.10.3/bin/kromarec --baseline-schema 9 --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/rooms_smoke.py bin/kromarec --baseline /path/to/0.10.1/bin/kromarec --baseline-schema 8
python3 tools/rooms_smoke.py bin/kromarec --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu --load-players 20
python3 tools/memory_rss.py bin/kromarec
```

Supply `--reference /path/to/pinned/recflare` to route_audit.py to check upstream source locations. For encrypted NameServer coverage, run `python3 /path/to/KromaRec-Patch/tools/encrypted_luxon_probe.py .` with that checkout's test-only Python cryptography dependency. The backend itself has no such dependency.

## Deployment and remaining blockers

Use [installation](INSTALL.md), [verified backup procedure](BACKUP.md) and [upgrade and rollback](UPGRADE.md). Keep the gate on 127.0.0.1:8081, backend behind Caddy on 127.0.0.1:8080, and the previous release plus private configuration for rollback. Set the existing signing secret with the DB path. Do not rotate it as part of this update.

The next native test starts at discovery TLS on one Windows PC, then captures the first failed HTTP request with sensitive fields removed. The device-ID grant contract, native asset/scene loading, Unity actor replication and voice remain unresolved. See [native-client-0.10.4.md](../compat/native-client-0.10.4.md) and [multiplayer.md](../compat/multiplayer.md). A browser error on a service root alone does not identify a client startup dependency.

## Changed files relative to the source distribution

- `CHANGELOG.md`
- `Cargo.lock`
- `Cargo.toml`
- `README.md`
- `SHA256SUMS`
- `bin/kromarec`
- `bin/kromarec-x86_64-linux-gnu`
- `compat/api-errors-0.10.4.md`
- `compat/multiplayer.md`
- `compat/native-client-0.10.4.md`
- `compat/route-status.tsv`
- `compat/routes-0.10.4.json`
- `compat/routes-0.10.4.md`
- `compat/service-hosts-0.10.4.md`
- `compat/update-matrix.py`
- `config/server.env.example`
- `docs/INSTALL.md`
- `docs/UPGRADE.md`
- `docs/verification-0.10.4.md`
- `packaging/kromarec.confd`
- `src/accounts.rs`
- `src/main.rs`
- `src/settings.rs`
- `src/social.rs`
- `tools/release_checks.py`
- `tools/rooms_smoke.py`
- `tools/route_audit.py`
- `tools/session_load.py`
- `tools/smoke.py`
- `tools/startup_probe.py`
