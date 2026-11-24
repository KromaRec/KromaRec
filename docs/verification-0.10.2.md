# KromaRec 0.10.2 compatibility verification

Source baseline is KromaRec commit 1d8558a6a46d93989ed77c59dc6db169d1b4a31d. The previous [33-service audit](API-AUDIT.md) records 0.10.1 behavior. The reference is [RecFlare 0.0.10](https://github.com/recflare/server/tree/86bdd02063629677f120916d5d419440e0aeee71). Route file names and line numbers are recorded in compat/route-status.tsv. The bundled JSON catalogues come from that commit; the copy of its MIT license is in compat/catalogue/LICENSE.

## Read contracts

All entries below use GET, with no request body. The suite checks real host routing, populated records, absent records, no-token and expired-token cases. The method guard preserves the dispatcher's HEAD behavior. Only the two indicated feature endpoints require an access bearer.

| Service / path | Request and returned data | Evidence about when it is called |
|---|---|---|
| discovery /sections/bulk | Public. Repeated id parameters filter the pinned sections by string ID in catalogue order. Duplicates collapse and missing IDs contribute nothing. A request with no IDs returns an empty array. | Reference tags 2025 discovery pages. |
| lists /curatedlists | Public. creatorAccountId, type and name. A matching stored player list wins, followed by name/type captures, reserved empty playlists and the reference miss object. Exact numeric ListId, ItemIds array and list metadata. | Reference tags 2025 Play pages and saved lists. |
| roomcomments /comments/get/{roomId} | Public. Returns stored DTOs newest first. Optional subRoomId, exclusive minId and count (default 100, bounded to 1–500). Coordinates are numbers. Unknown rooms return an empty array. | Reference describes a room-load read. |
| clubs /club/{numericId} | Public. Selects a stored club and projects its public PascalCase DTO. Missing record returns 404. Internal fields are omitted. | Club profile lookup. |
| studio /cloud-builds | Public build metadata lookup by positive decimal cloudBuildId. Returns a success/error/value envelope, 400 for malformed IDs and 404 for absent records. Completed successful builds include the server-data filename. | Studio workflow. No build worker is required by this read. |
| ai /gameai/user/access | Bearer required. Optional roomId. Returns success false with error_id AI.RoomDoesNotSupportGameAI because the deployment offers no Game AI. Anonymous requests receive 401. | Reference tags 2025. |
| commerce /purchase/v1/hasspentmoney | Public. Returns JSON boolean false on this deployment, which has no real-money transactions. | Reference documents the consumer treating 404 as an error. |
| cards /iam/me/channels/{channel} | Bearer required. None, Announcements and JustInTimeTutorials return the empty campaign-message array on this deployment. Unknown channels return 400 after authentication. | Consumer expects an array of in-app messages. |
| api /iam/me/channels/{type} | Public local service without external account-channel links. Any single type returns an empty array. Cards authentication still applies to the Cards hostname. | Reference account handler explicitly describes a lookup during account loading. |

Native acceptance is blocked. No supported Windows client installation or real 20230414 trace was available. None of these reads is confirmed necessary for native startup. Two simulated peers and a successful Orientation metadata read do not establish Unity scene loading, spawning or movement. Full club management, comment creation, playlist editing and Studio execution remain outside this read release. Tables start empty. Tests insert disposable nonempty fixtures into SQLite to exercise actual lookup behavior. No fixture accounts, rooms, credentials or databases ship.

## Running verification locally

Use the extracted repository as the working directory. Run python3 tools/smoke.py bin/kromarec and python3 tools/service_hosts_test.py bin/kromarec --caddy caddy. The HTTPS suite uses a local wildcard certificate and a loopback TCP destination, retaining each hostname for SNI and the HTTP Host header. Private LAN DNS and client trust require the Windows test in the previous audit. Use tools/https_proxy_smoke.py for the startup and WSS suite. For the Photon tests use tools/rooms_smoke.py with --secure-luxon pointing at the supplied glibc binary on a compatible host or a patched native build on Alpine. The public UDP server ports and private TCP gate are separate.

## Release tests

Both Rust unit tests passed on each optimized target using Rust 1.100.0 and Cargo.lock. Both binaries passed smoke.py including catalogue_checks.py. Migration and room tests passed for schema 8 → 9 on musl and 7 → 9 on GNU. Tests preserved accounts and rooms, checked cursor filtering and confirmed local-owner list precedence. The musl run with patched Luxon passed synthetic NameServer → Master → Game authentication, invalid and expired tickets, private and banned joins, active eviction, two-way events, leave/rejoin and transport failure tests. HTTPS bootstrap, profile and settings flows, Orientation metadata, CDN loading tips, SignalR negotiation and authenticated WSS passed in automated tests. Booted OpenRC was unavailable. The Linux backend was tested on Debian x86-64. The supplied Luxon executable requires glibc; Alpine deployment uses the supplied patch and native build procedure.

| Metric | 0.10.1 | 0.10.2 |
|---|---:|---:|
| Median musl idle RSS, KiB | 3,700 | 3,788 |
| Median warm RSS with 100 HTTP keep-alives, KiB | 6,332 | 6,388 |
| Two-peer backend / Luxon RSS, KiB | 3,880 / 3,916 | 3,952 / 3,724 |
| Combined two-peer snapshot, KiB | 7,796 | 7,676 |
| Idle process CPU over 2 s | 0.000 s | 0.000 s |
| HTTP workload process CPU | 0.420 s | 0.410 s |

HTTP medians are from three runs. The updated suite exercises more routes. The two-peer sample is a point-in-time observation. These figures do not establish a performance improvement.

## All 33 service results

Direct HTTP and CA-verified HTTPS agreed on response status and type. Private .rec.test DNS was unavailable in this environment. The harness retained TLS hostname validation while directing each TCP connection to local Caddy.

| Host | GET / | Representative anonymous request |
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

## Changed files relative to 0.10.1

- `CHANGELOG.md`
- `Cargo.lock`
- `Cargo.toml`
- `README.md`
- `SHA256SUMS`
- `bin/kromarec`
- `bin/kromarec-x86_64-linux-gnu`
- `bin/luxon-kromarec-x86_64-linux-gnu`
- `compat/catalogue/LICENSE`
- `compat/catalogue/curated-lists.json`
- `compat/catalogue/sections.json`
- `compat/route-status.tsv`
- `compat/update-matrix.py`
- `docs/INSTALL.md`
- `docs/UPGRADE.md`
- `docs/verification-0.10.2.md`
- `migrations/009.sql`
- `src/catalogue.rs`
- `src/main.rs`
- `src/settings.rs`
- `tools/catalogue_checks.py`
- `tools/memory_rss.py`
- `tools/rooms_smoke.py`
- `tools/service_hosts_test.py`
- `tools/smoke.py`
