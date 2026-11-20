# KromaRec 0.5 verification

## Results

| Stage | Result |
| --- | --- |
| KromaRec 0.4 → 0.5 migration | Existing account, settings and room survived. SQLite foreign-key and integrity checks passed. |
| Startup wire behavior | Verified local credentials, unlinked platform picker, Orientation on first signup, early logout handling, transition to dorm, and bulk room resolution. |
| Room and SignalR flow | Two HTTP accounts created/discovered and entered the same room. The suite checked instance ID, room name, scene and region, permissions, invites, occupancy, and notification delivery. |
| Luxon 1.5 protocol | Two loopback wire peers used the app ID and named room allocated by KromaRec. Both went through real ENet/GP Binary NameServer, Master and Game authentication, joined with distinct actor IDs, and exchanged custom events. Player B left, A received the leave event, and B rejoined and sent another event. |
| Rec Room 20230414 | Pending. The environment had neither an installed 20230414 game nor two configured game-client sessions. The tests use Photon operations and arbitrary test payloads. Unity scene loading, movement, gestures, game RPCs and voice were not exercised. |

The same suites passed using the x86-64 musl and glibc release binaries on
Debian 13. Luxon was the official 1.5 static x86-64 release. There were no
Rust runtime dependencies added in 0.5. Alpine/OpenRC remains untested.

## Authentication boundary

The wire test first authenticates to Luxon using its anonymous/weak app
configuration and then follows its opaque token handoff. The test had to
send ENet disconnect on each prior peer before authenticating to the next
server, allowing the stock Luxon handler to release and store its peer state.
RecNet's `photonAuthToken` is a distinct JWT. Stock Luxon looks up its own
handoff tokens and does not validate that JWT or check SQLite authorization
for a named room. Its optional custom authentication requires a plugin-enabled
build and app/provider setup, plus verification of what the actual 20230414
client sends. Room-level authorization would additionally need enforcement
in Luxon's admission hooks. No such plugin is shipped.

Private room admission and invitation checks work over RecNet. Keep the
experimental Luxon deployment on a private network or limit UDP to the
participating clients. A client that can reach stock Luxon and knows the
room name can attempt to join without a RecNet reservation.

## Photon Voice

RecFlare 0.0.10 documents Photon Voice for 20230414. Luxon's Realtime
event test makes no assertion about the game's separate voice connection,
codec, encryption or permission behavior. The Realtime and Voice app IDs
remain separately configurable in KromaRec. Tachyon has not been added.

## Memory measurements

RSS in KiB, medians of three sequential runs on the same host. Warm uses
the regression suite with 100 open HTTP keep-alives.

| Static build | Binary bytes | Idle RSS | Warm RSS |
| --- | ---: | ---: | ---: |
| 0.4 | 3,313,056 | 3,220 | 5,912 |
| 0.5 | 3,341,728 | 3,312 | 6,008 |
| Change | +28,672 | +92 | +96 |

The separate room and transport integration run recorded 3,564 KiB KromaRec
RSS with one SignalR connection open. Photon traffic runs in a separate
process. These snapshots exclude peak password hashing and Unity clients.
