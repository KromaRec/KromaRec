# KromaRec 0.10.3 verification

Native Rec Room 20230414 multiplayer compatibility remains unverified.

The release starts from the 0.10.2 source and assets in the distributed Linux ZIP (SHA-256 1e7abc6a38508fca3aedb91079f21734f4fb50d59da8d1988f89be4cdcf3d978). At inspection the GitHub default branch declared 0.10.1. The reference checkout was RecFlare 86bdd02063629677f120916d5d419440e0aeee71. The Linux workspace had no authorized Windows game installation or generated client interop to run.

## Confirmed corrections

| Interface | 0.10.2 observation | 0.10.3 result |
|---|---|---|
| GET ns / and configured service origins | A CDN override of cms let two directory entries claim the same HTTPS origin. Collision detection covered only a subset of services. | Startup rejects ambiguous names, case-insensitively, across all 33 entries and ns before opening SQLite. Default routing still passes. |
| GET api /api/versioncheck/v4 and POST auth /connect/token | A 20230414 version with an excessively long point suffix received VersionStatus 0 while the token handler rejected it. | Both apply the existing 32-byte supported version rule. The regular 20230414.01 version remains supported. Explicit invalid versions receive VersionStatus 1 and invalid_grant. |
| Client-side HTTPS preflight | DNS failure, connection refusal and an untrusted test CA all produced URLError. | Cause-specific errors omit the URI and payload. HTTP success is printed separately from passing the bootstrap contract checks. |

These are configuration and protocol regressions reproduced in a test environment. There is no captured failed native request in this run. The inventory still contains 689 routes.

## Results

| Status | Check | Evidence |
|---|---|---|
| PASS | Static musl and dynamic glibc x86-64 | Rust 1.100.0, optimized locked builds. Two unit tests passed per target. Both release binaries passed HTTP smoke. |
| PASS | HTTPS and routing | All 33 service origins through local Caddy with trusted test CA, SNI and original Host. Direct HTTP status and type agreed. Negative alias cases and public gate bind rejection passed. See the host rows below. |
| PASS | Startup | Version, discovery, accounts, config, loading tips, room and subroom scene IDs, matchmaking and authenticated SignalR WSS. |
| PASS | SQLite and accounts | Schema 9 from a disposable 0.10.2 database retained its profile, password login, refresh credential, settings and room after upgrade. Schema 8 to 9 and restart persistence passed separately. No migration added. |
| PASS | Photon | Patched Luxon 1.5 NameServer region discovery, Master and Game handoffs, private room checks, bans, invalid and expired tickets, active eviction, leave, disconnect, reconnect and shutdown. Tests used ephemeral loopback ports. Deployment gate is 127.0.0.1:8081. |
| PASS | Scene admission | One new synthetic account entered the assigned Orientation Game room, left, matched to its dorm and entered that room. Each match response agreed with room metadata and connection-info. |
| PASS | Ten simultaneous synthetic players | Ten separate authenticated accounts reserved the same RecCenter instance and received distinct actor numbers. Each sent ten broadcasts. All 900 expected deliveries matched sender and payload, then all actors left. |
| BLOCKED | Native startup and multiplayer | No Windows game or generated interop on the test host. Native HTTP request sequence, encrypted game handshake, dorm rendering, movement, gestures and reconnect require observations on two clients. |
| NOT TESTED | Operator's network and booted Alpine/OpenRC | Local TLS checks override only the TCP destination. Windows DNS and CA trust, OpenRC startup on a booted Alpine installation, packet loss and WAN stability require deployment testing. |

The bundled glibc Luxon is the same patched executable tested in 0.10.2. Its pinned build provenance is in compat/verification-0.10.1.md, with license notices in packaging/luxon. This run rebuilt both Rust binaries and exercised the bundled Luxon at the transport boundary.

## Resource measurements

KiB of resident memory on Linux x86-64, using static musl KromaRec and glibc Luxon. Idle and warm HTTP numbers are medians of three runs. Photon RSS is a snapshot during the indicated workload. CPU seconds are process counters with scheduler tick granularity.

| Workload | 0.10.2 | 0.10.3 |
|---|---:|---:|
| Idle backend, published baseline | 3788 | 3804 |
| Warm backend, 100 HTTP keep-alives | 6388 | 6396 |
| Idle CPU over 2 s / HTTP workload CPU, seconds | 0.000 / 0.410 | 0.000 / 0.450 |
| Two peers, backend + Luxon, same extended suite | 3644 + 3836 = 7480 | 3904 + 3892 = 7796 |
| Ten peers, backend + Luxon, same extended suite | 3684 + 5332 = 9016 | 3928 + 5368 = 9296 |
| Two peers, 15 bidirectional exchanges over 15 s, backend / Luxon CPU seconds | 0.010 / 0.010 | 0.000 / 0.010 |
| Ten peers, 900 deliveries, wall seconds | 10.795 | 10.654 |
| Ten peers, backend / Luxon CPU seconds | 0.010 / 0.030 | 0.000 / 0.020 |

The earlier published two-peer combined snapshot was 7676 KiB. These short loopback runs establish neither peak memory nor a performance improvement. The ten-peer test includes a one-second pause per broadcast round. Zero CPU seconds means no measurable counter increment in that interval.

## Reproduce

Run from an extracted package in a clean test environment with Python 3.11+, Caddy and glibc for the supplied Luxon binary.

```sh
sha256sum -c SHA256SUMS
python3 tools/smoke.py bin/kromarec
python3 tools/smoke.py bin/kromarec-x86_64-linux-gnu
python3 tools/service_hosts_test.py bin/kromarec --caddy caddy
python3 tools/https_proxy_smoke.py bin/kromarec --caddy caddy
python3 tools/rooms_smoke.py bin/kromarec --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
python3 tools/memory_rss.py bin/kromarec
```

Add --baseline /path/to/0.10.2/kromarec --baseline-schema 9 to the rooms command for the upgrade test. The tests create temporary databases. Installation and rollback are in docs/INSTALL.md, docs/BACKUP.md and docs/UPGRADE.md.

## First native run

1. From each Windows PC, verify LAN DNS for ns.rec.test and the advertised services and install the public Caddy root CA. Run the preflight from the backend package:
   `py -3 tools/https_preflight.py https://ns.rec.test --match-url https://match.rec.test`.
2. Use the authorized Steam 20230414 depot. Follow KromaRec-Patch/NEXT_STEPS.md to verify the manifest, generate interop through a normal launch and run the patch readiness check. Use separate local accounts. Point the client's NameServer at the LAN server and use the server's region and app ID. Keep the original security checks enabled.
3. Enable KROMAREC_TRACE_STARTUP=true temporarily and correlate one native launch with the patch readiness report. Record UTC time, service, method, route template, status, expected structure, observed field names/types and sanitized error. Use fixed route labels for account and room identifiers. Keep headers, cookies, query values, tokens, device IDs and raw bodies out of the report.
4. The pinned reference describes POST /api/PlayerReporting/v1/deviceId as unresolved during account creation. KromaRec returns 404 in the synthetic test. A successful native acknowledgement contract is unknown. Record the first actual failure and last completed screen.
5. The 0.3.1 patch preview gates native Photon auth pending verification of obfuscated members. Follow KromaRec-Patch/PHOTON_DIAGNOSTICS.md to verify the exact interop members and parameter 216 serialization, encryption and handoff before changing the patch. Keep Luxon admission enabled. No backend evidence justifies inventing new movement events.
6. Once client A reaches a rendered scene, repeat for client B. Use compat/multiplayer.md to compare instance, scene, region and actor identity, witness bidirectional movement and an interaction on both screens, then have B leave and rejoin while A stays. Save the sanitized timeline and connection outcomes.

## 33-host results

Each representative result below passed direct HTTP and local CA-verified HTTPS. An anonymous authorization error, an absent record or an unimplemented route can be the expected result.

| Host | GET / | Representative request |
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

## Changed files from the distributed 0.10.2 archive

- `CHANGELOG.md`
- `Cargo.lock`
- `Cargo.toml`
- `README.md`
- `SHA256SUMS`
- `bin/kromarec`
- `bin/kromarec-x86_64-linux-gnu`
- `docs/INSTALL.md`
- `docs/UPGRADE.md`
- `docs/verification-0.10.3.md`
- `src/auth.rs`
- `src/bootstrap.rs`
- `src/main.rs`
- `tools/https_preflight.py`
- `tools/https_proxy_smoke.py`
- `tools/rooms_smoke.py`
- `tools/service_hosts_test.py`
- `tools/session_load.py`
- `tools/smoke.py`
