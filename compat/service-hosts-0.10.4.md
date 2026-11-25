# HTTPS service audit, KromaRec 0.10.4

PASS for all 33 advertised HTTPS origins and the separate NameServer. Each TLS request used the original SNI hostname and a trusted ephemeral Caddy CA. The test directed TCP to loopback. The host OS resolver did not resolve these private names. Production DNS, client trust and LAN routing remain NOT TESTED. Each service root matched direct HTTP byte-for-byte, including status and MIME. Negotiation IDs vary per call and were checked structurally.

| Host | Pinned rows | GET / | MIME | Exact root body | Anonymous method/path probe |
|---|---:|---:|---|---|---|
| ai.rec.test | 9 | 200 | application/json | `{"service":"ai","status":"ok"}` | GET /gameai/user/access → 401 |
| api.rec.test | 195 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | GET /api/versioncheck/v4?v=20230414 → 200 |
| accounts.rec.test | 25 | 200 | application/json | `{"service":"accounts","status":"ok"}` | GET /account/me → 401 |
| auth.rec.test | 19 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | POST /connect/token → 415 |
| cdn.rec.test | 14 | 200 | application/json | `{"service":"cdn","status":"ok"}` | GET /config/LoadingScreenTipData → 200 |
| cms.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| cards.rec.test | 2 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /iam/me/channels/Announcements → 401 |
| chat.rec.test | 28 | 200 | application/json | `{"service":"chat","status":"ok"}` | GET /thread/1/message → 404 |
| clubs.rec.test | 40 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | GET /club/1 → 404 |
| commerce.rec.test | 10 | 200 | application/json | `{"service":"commerce","status":"ok"}` | GET /purchase/v1/hasspentmoney → 200 |
| data.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| discovery.rec.test | 4 | 200 | application/json | `{"service":"discovery","status":"ok"}` | GET /sections/bulk → 200 |
| econ.rec.test | 99 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | GET /api/influencerpartnerprogram/influencers → 404 |
| geo.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| img.rec.test | 3 | 200 | application/json | `{"service":"img","status":"ok"}` | GET /openapi.json → 404 |
| leaderboard.rec.test | 6 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /openapi.json → 404 |
| link.rec.test | 2 | 200 | text/plain; charset=utf-8 | `hello, world!` | POST /actionlink → 404 |
| lists.rec.test | 8 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /curatedlists → 200 |
| match.rec.test | 40 | 401 | absent | empty | GET /player/connection-info → 401 |
| moderation.rec.test | 2 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /voice/config → 200 |
| notify.rec.test | 8 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | POST /hub/v1/negotiate?negotiateVersion=1 → 200 |
| platformnotifications.rec.test | 5 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /preferences → 404 |
| playersettings.rec.test | 5 | 200 | application/json | `{"service":"playersettings","status":"ok"}` | GET /playersettings → 401 |
| roomcomments.rec.test | 4 | 200 | application/json | `{"service":"roomcomments","status":"ok"}` | GET /comments/get/1 → 200 |
| roomieintegrations.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| rooms.rec.test | 84 | 200 | application/json | `{"service":"rooms","status":"ok"}` | GET /rooms/13 → 200 |
| storage.rec.test | 5 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /openapi.json → 404 |
| strings.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| strings-cdn.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| studio.rec.test | 23 | 200 | text/plain; charset=utf-8 | `hello, world!` | GET /cloud-builds → 400 |
| thorn.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| videos.rec.test | 0 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | No non-root route in pinned inventory |
| www.rec.test | 44 | 404 | application/json | `{"success":false,"error":{"message":"not found"}}` | GET /privacy → 404 |
| ns.rec.test | 1 | 200 | application/json | 33 distinct configured HTTPS origins | GET / validated against the service mapping |

Authentication, valid bodies and mutation side effects are exercised separately by the smoke and room suites. Full inventory and reference file/line are in [routes-0.10.4.json](routes-0.10.4.json).
