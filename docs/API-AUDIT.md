# KromaRec 0.10.1 service routing audit

The test fetched the service directory at ns.rec.test and exercised all 33 returned HTTPS origins against the release binary and Caddy 2.6.2. Each request also ran directly against Axum with the same Host, method and path. Tests used a temporary SQLite database and synthetic accounts. The HTTPS probes verified the hostname and certificate chain using a private test CA and routed TCP to a local Caddy instance. Ordinary DNS lookup in the build environment failed for all 33 private names. DNS, Windows certificate trust and native gameplay on the operator's network remain unverified.

The baseline is the previous precompiled 0.10.1 discovery update. The table shows anonymous GET / before and after. All 33 HTTPS handshakes passed. Direct and proxy responses matched for status, type and body, with SignalR negotiation tested by response structure because it issues fresh connection tokens. Distinct host-specific health responses, the ns directory, startup paths and negative controls exercise Host forwarding; identical fallback bodies alone cannot establish service identity. No redirects were observed. Every host was also tested with an unknown path. A 401 indicates an authentication boundary, 415 indicates the token endpoint rejected the probe's missing form media type, and 404 means the requested resource did not resolve. The route inventory covers source declarations and cannot establish which calls a particular Windows client will make.

| Service / hostname | HTTPS | Routing GET / (before → after) | Non-root endpoint and result | Interpretation |
|---|---|---|---|---|
| AI / `ai.rec.test` | TLS OK | 404 → 200 | GET /gameai/user/access → 404 | Verified upstream liveness handler added |
| API / `api.rec.test` | TLS OK | 404 → 404 | GET /api/versioncheck/v4?v=20230414 → 200 | Service uses specific paths and methods |
| Accounts / `accounts.rec.test` | TLS OK | 200 → 200 | GET /account/me → 401 | Existing public health handler |
| Auth / `auth.rec.test` | TLS OK | 404 → 404 | POST /connect/token → 415 | Service uses specific paths and methods |
| CDN / `cdn.rec.test` | TLS OK | 200 → 200 | GET /config/LoadingScreenTipData → 200 | Existing public health handler |
| CMS / `cms.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Cards / `cards.rec.test` | TLS OK | 404 → 200 | GET /iam/me/channels/Announcements → 404 | Verified upstream liveness handler added |
| Chat / `chat.rec.test` | TLS OK | 404 → 200 | GET /thread/1/message → 404 | Verified upstream liveness handler added |
| Clubs / `clubs.rec.test` | TLS OK | 404 → 404 | GET /club/1 → 404 | Root and selected path unavailable |
| Commerce / `commerce.rec.test` | TLS OK | 404 → 200 | GET /purchase/v1/hasspentmoney → 404 | Verified upstream liveness handler added |
| Data / `data.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Discovery / `discovery.rec.test` | TLS OK | 200 → 200 | GET /sections/bulk → 404 | Existing public health handler |
| Econ / `econ.rec.test` | TLS OK | 404 → 404 | GET /api/influencerpartnerprogram/influencers → 404 | Root and selected path unavailable |
| Geo / `geo.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Images / `img.rec.test` | TLS OK | 404 → 200 | GET /openapi.json → 404 | Verified upstream liveness handler added |
| Leaderboard / `leaderboard.rec.test` | TLS OK | 404 → 200 | GET /openapi.json → 404 | Verified upstream liveness handler added |
| Link / `link.rec.test` | TLS OK | 404 → 200 | POST /actionlink → 404 | Verified upstream liveness handler added |
| Lists / `lists.rec.test` | TLS OK | 404 → 200 | GET /curatedlists → 404 | Verified upstream liveness handler added |
| Matchmaking / `match.rec.test` | TLS OK | 401 → 401 | GET /player/connection-info → 401 | Matchmaking requires credentials |
| Moderation / `moderation.rec.test` | TLS OK | 200 → 200 | GET /voice/config → 200 | Existing public health handler |
| Notifications / `notify.rec.test` | TLS OK | 404 → 404 | POST /hub/v1/negotiate?negotiateVersion=1 → 200 | Service uses specific paths and methods |
| PlatformNotifications / `platformnotifications.rec.test` | TLS OK | 404 → 200 | GET /preferences → 404 | Verified upstream liveness handler added |
| PlayerSettings / `playersettings.rec.test` | TLS OK | 200 → 200 | GET /playersettings → 401 | Existing public health handler |
| RoomComments / `roomcomments.rec.test` | TLS OK | 404 → 200 | GET /comments/get/1 → 404 | Verified upstream liveness handler added |
| RoomieIntegrations / `roomieintegrations.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Rooms / `rooms.rec.test` | TLS OK | 404 → 200 | GET /rooms/13 → 200 | Verified upstream liveness handler added |
| Storage / `storage.rec.test` | TLS OK | 404 → 200 | GET /openapi.json → 404 | Verified upstream liveness handler added |
| Strings / `strings.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| StringsCDN / `strings-cdn.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Studio / `studio.rec.test` | TLS OK | 404 → 200 | GET /cloud-builds → 404 | Verified upstream liveness handler added |
| Thorn / `thorn.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| Videos / `videos.rec.test` | TLS OK | 404 → 404 | No non-root route in pinned inventory | Advertised only in this inventory |
| WWW / `www.rec.test` | TLS OK | 404 → 404 | GET /privacy → 404 | Root and selected path unavailable |

## Response contracts and verified fixes

Six previously missing roots return JSON with the canonical service name and status ok (ai, chat, commerce, img, roomcomments, rooms). Seven return the reference text/plain greeting (cards, leaderboard, link, lists, platformnotifications, storage, studio). Discovery remains a separate JSON health endpoint. Only GET and the existing HEAD handling receive these public responses. Protected subpaths continue through their original checks.

Overrides now route the newly covered services and the discovery worker to their advertised aliases. API config generates ShareBaseUrl using the configured www alias. Tests cover non-default aliases for AI, discovery, player settings and www. Collisions between implemented service aliases remain unsupported and are rejected at startup. The default Caddy example preserves Host and proxies only the RecNet listener. No Caddy or database migration change was required.

## Reference handler evidence
- ai: `{"service":"ai","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/ai/src/ai.app.ts#L90)
- cards: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/cards/src/cards.app.ts#L23)
- chat: `{"service":"chat","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/chat/src/chat.app.ts#L664)
- commerce: `{"service":"commerce","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/commerce/src/commerce.app.ts#L48)
- img: `{"service":"img","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/img/src/img.app.ts#L342)
- leaderboard: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/leaderboard/src/leaderboard.app.ts#L132)
- link: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/link/src/link.app.ts#L39)
- lists: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/lists/src/lists.app.ts#L609)
- platformnotifications: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/platformnotifications/src/platformnotifications.app.ts#L57)
- roomcomments: `{"service":"roomcomments","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/roomcomments/src/roomcomments.app.ts#L80)
- rooms: `{"service":"rooms","status":"ok"}` (application/json). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/rooms/src/rooms.app.ts#L1382)
- storage: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/storage/src/storage.app.ts#L247)
- studio: `hello, world!` (text/plain; charset=utf-8). [Pinned handler](https://github.com/recflare/server/blob/86bdd02063629677f120916d5d419440e0aeee71/apps/studio/src/studio.app.ts#L189)

## Startup, auth and scope

The HTTPS bootstrap suite accepted build 20230414, loaded API/game configuration, emoji configuration, CDN loading tips and the correct Orientation scene ID, and negotiated SignalR. HTTPS signup and the authenticated WSS handshake/ping passed. The HTTP regressions exercised password login, token rotation and revocation, settings and access control. The protected match connection-info probe returned 401 anonymously. The room suite exercises the authenticated flow and signed admission at the actual patched Photon transport. Reaching a service root does not prove chat, storage, commerce, or leaderboard functionality.

The pinned matrix contains 689 declarations. See compat/route-status.tsv for their individual implementation status and source locations. Representative missing paths above are coverage gaps. Native startup necessity has not been demonstrated for them. Ads-only hosts have no non-root declaration in this inventory. Additional unresolved points include native platform login/device-ID behavior, real client scene and asset requirements, and gameplay replication. No credentials were sent to third parties. Live public discovery could not be retrieved in the audit environment. Comparisons use pinned RecFlare 0.0.10 source.

## Run the tests

```sh
python3 tools/service_hosts_test.py bin/kromarec --caddy caddy
python3 tools/https_proxy_smoke.py bin/kromarec --caddy caddy
python3 tools/smoke.py bin/kromarec
```

For real LAN DNS and TLS verification, run this PowerShell block in the extracted repository on the Windows test PC. It uses the shipped service mapping and the PC's configured DNS and trust store. It continues if one host fails. HTTP 000 with a curl error means no HTTP status was received.

```powershell
$services = Get-Content ./compat/service-hosts.json -Raw | ConvertFrom-Json
foreach ($s in $services.PSObject.Properties) {
    if ($s.Name -in 'GameLogs','DataCollection','BugReporting') { continue }
    $name = "$($s.Value).rec.test"
    curl.exe -sS --noproxy "*" --max-time 5 -o NUL -w "$name HTTP %{http_code} %{content_type}\n" "https://$name/"
}
```

Then run tools/https_preflight.py against https://ns.rec.test with --match-url https://match.rec.test. For native acceptance follow compat/multiplayer.md and capture the first failed host, method and path with credentials and identifiers removed.

## Release verification results

- Rust 1.100.0, locked optimized builds for x86_64-unknown-linux-musl and x86_64-unknown-linux-gnu. Both x86-64 ELFs started. The musl binary has no ELF interpreter or NEEDED libraries.
- Two Rust unit tests passed on each target. Both binaries passed the HTTP/auth smoke suite. On musl the HTTPS/Caddy/WSS suite and all 33 service checks passed. Gate startup rejected a wildcard bind.
- Schema 7 from the 0.9 package migrated to 8 on musl. The GNU build reopened schema 8 from the 0.10.0 package. Room allocation, privacy, party invitations, persistence across restart and SignalR passed.
- The bundled glibc Luxon passed Name → Master → Game, invalid/expired/revoked admission, two-peer events, leave/rejoin, active eviction and fail-closed disconnection. New Linux builds were tested on Debian x86-64. A booted Alpine/OpenRC deployment and native Windows clients were unavailable for this build.
- Three-run median KromaRec musl RSS: 3,700 KiB idle; 6,332 KiB with 100 HTTP keep-alives. The two-peer sample was 3,880 KiB KromaRec plus 3,916 KiB Luxon (7,796 KiB combined). These are RSS snapshots. Median backend CPU over the two-second idle interval was 0.000 s. Median backend CPU during the HTTP workload was 0.420 s.
- Changed backend source: src/main.rs. Changed or added checks and inventory: tools/smoke.py, tools/service_hosts_test.py, compat/update-matrix.py, compat/route-status.tsv. Source reference and base: pinned RecFlare 0.0.10 and the previous KromaRec 0.10.1 discovery precompiled package.
