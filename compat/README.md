# RecFlare 0.0.10 compatibility audit

[Source at the pinned commit](https://github.com/recflare/server/tree/86bdd02063629677f120916d5d419440e0aeee71) · [Services](https://github.com/recflare/server/blob/0.0.10/SERVICES.md) · [Deployment](https://github.com/recflare/server/blob/0.0.10/DEPLOYING.md)

`upstream-routes.tsv` records 689 literal method/path declarations and router mounts extracted from the tagged TypeScript AST, with service, file and line. It includes public APIs, placeholder routes, web pages, diagnostics and OpenAPI documents. Alias arrays are expanded. This is a source inventory. It does not enumerate paths synthesized by framework middleware or derived from static files. Use the source lines and upstream fixtures to establish per-route contracts.

| Feature | Upstream sources | Current KromaRec coverage |
| --- | --- | --- |
| Service discovery and startup | ns, api, accounts, datacollection | Discovery, startup config, version checks for 20230414 and same-day revisions, emoji list and disabled analytics configuration |
| Account management, OAuth, sessions and profiles | accounts, auth, api | Local password signup, password login, refresh rotation, and basic public/private profile reads. Password changes and selected profile edits work. Platform credentials, recovery and deletion are pending. Lobby logout clears presence. |
| Settings | playersettings | Read, merge and delete settings in SQLite. Defaults, JSON/form payloads and ignored keys are preserved. The OpenAPI route is pending. |
| Friends, clubs and discovery | api, clubs, discovery, lists, roomcomments | Friendship lifecycle, flags, mutual friends and live lobby/room counts work over HTTP. Friends can read redacted room locations. Clubs are pending. |
| Rooms, presence and matchmaking | match, rooms, api | Persistent room CRUD, template cloning, room roles, ownership transfer, dorm provisioning, transactional allocation, party invitations, instance matchmaking policy and presence work over HTTP. Room saves, extra subrooms, bans, event matchmaking and newer-build wire variants remain pending. |
| Economy, commerce and inventory | econ, commerce, api | Pending |
| Chat and notifications | chat, notify, platformnotifications | Durable friendship and co-owner invite messages. Live room, presence and invite notifications work over SignalR. General chat, channel subscriptions and the offline hub queue are pending. |
| Leaderboards and progression | leaderboard, api | Pending |
| Moderation and administration | moderation, api, accounts, auth, www | Voice-moderation config only |
| Asset storage and delivery | cdn, img, storage, studio, api | Local CDN config, signature, room, invention, data, avatar and room-authoring downloads. Uploads, image processing, videos and Unity catalog lookups pending. |
| Other workers | ai, cards, link, mono, www | Pending. Upstream monolith shares the same paths for the services it mounts. |

## Client compatibility

| Build | Status |
| --- | --- |
| 20230414 | HTTP and SignalR integration tests. Actual client authentication, scene loading, Photon gameplay and voice unverified. |
| 20250718 and other 2025 builds | Pending. |
| 20260323 | Pending. |

## Data model for remaining services

Reuse the pinned D1 migrations as input for the game schema, with a dedicated import path for existing D1/KV data. The settings schema in this build is KromaRec-specific. Store persistent settings in SQLite rows rather than copying whole KV documents on update. Map R2 objects to locally stored files with ownership and metadata in SQLite. Add upload authorization before exposing writable storage. The in-process WebSocket hub uses bounded live buffers. Friendship and role-invite messages persist in the SQLite inbox. Redis and PostgreSQL are unnecessary for the current implementation.

## Multiplayer and external services

| Component | Role at the pinned tag | KromaRec status |
| --- | --- | --- |
| Matchmaking worker | Owns presence, room allocation, join rules and game-facing response envelopes. `GET /player/connection-info` carries the selected Photon room ID, region, app IDs and custom auth token. KromaRec resolves the Photon room ID from the caller’s live instance only. | HTTP-tested for two accounts and capacity overflow. |
| Photon Realtime and Voice | The deployment guide calls for Photon Realtime and Voice for 2023. Photon transports realtime gameplay independently of the HTTP API. The upstream guide names Luxon and OpenPhoton as custom server candidates. | Two wire peers completed the patched Luxon 1.5 NameServer → Master → Game chain with verified identities and room admission, relayed events, and rejoined. Rec Room gameplay, Photon Voice and OpenPhoton remain unverified. |
| Photon Chat | The upstream connection document advertises a distinct Photon Chat app ID. The RecFlare HTTP chat worker is separate. | Pending protocol and client validation |
| Tachyon | Voice for the 2025-era stack. The match worker can select a Tachyon host per room instance from configured pools, with empty voice fields when no server is available. | Outside the primary 20230414 test target. No deployment or resource measurement. |
| QoS probes | The tagged backend advertises external probe addresses while pinning the session region by configuration. | Pending. A fully local deployment will require operator-controlled probes or validation that probes can be omitted. |

A multiplayer acceptance run needs two patched 20230414 clients, a populated room dataset, and an operator-controlled Photon-compatible server. Check token issuance, startup through dorm entry, public and private joins, room/presence IDs returned to both clients, movement replication, voice, reconnects and moderation using the client. Measure KromaRec, the TLS proxy and Photon/voice processes separately during those runs.

## Known wire differences

The matrix retains all 689 declarations, with per-route status in `route-status.tsv`. Explicitly disabled telemetry is tracked in its row notes.

The current binary intentionally gates version check to 20230414. CDN ETags derive from local file size and mtime and may differ from R2 values. The CDN handles single byte ranges. Malformed or unsatisfiable `bytes=` requests can yield 416, whereas some upstream invalid requests are normalized by R2. Asset configuration is served from operator-provided files. Discovery omits telemetry destinations. Unported routes normally return a 404. Signup is opt-in, local passwords use Argon2id, and the token endpoint currently supports only local password and refresh flows.
