# KromaRec 0.6 verification

## Results

| Layer | Verified |
| --- | --- |
| Rust core | Release suites passed on x86-64 static musl and glibc. Preserved the 0.5 HTTP, room, SignalR and stock Luxon protocol tests. |
| Database | Migrated an actual 0.5 database in place and preserved an existing account, settings and room. Passed WAL, foreign-key and integrity checks. |
| Startup | Synthetic tests covered password flow, the unlinked platform picker, Orientation, dorm allocation, room scene metadata, and lobby lock acquisition and verification. |
| Photon authentication | Real patched Luxon 1.5 rejected missing, corrupted, expired, wrong-app and RecNet bearer credentials. It replaced a spoofed user ID with the signed account ID. It rechecked credentials on native handoff. |
| Photon room admission | A valid account without an active reservation could not use a known private room name. A reservation revoked between the Master and Game operations failed at the Game endpoint. Expired reservations and banned players were denied. |
| Photon relay | Two wire peers used an allocated KromaRec room, traversed Name → Master → Game, got distinct actor IDs, relayed events in both directions, and exercised leave and rejoin. |
| Alpine | The static executable started in an Alpine 3.22.0 userspace chroot and served the version endpoint. OpenRC 0.62.6 installed, but the namespace could not mount /proc or assign the service UID, so service start/stop and Alpine Luxon remain unverified. |
| Actual Rec Room | No 20230414 game binary or two runnable clients were available. Manual-login UI, Unity scene rendering, actual movement, gestures, game RPCs, reconnect and voice remain unverified. |

## Security scope

Luxon 1.5 offers custom-auth providers in a plugin-enabled build.
Its room plugin callbacks are reached through the room's plugin list,
which is supplied by the requesting client. The extension instead makes
authorization mandatory in the central auth path and both named room
admission paths. It disables the unsolicited and random-join alternatives
for this dedicated deployment. It uses a keyed, bounded IPv4 loopback
channel, so Luxon does not need the RecNet signing key or SQLite access.
Failure or timeout denies the operation. It is not a general-purpose Luxon
build and uses a single process for native handoff state.

The permission decision holds at admission time. A ban or room deletion
does not forcibly evict a peer already in a Photon room. The local table
allows operator-managed transport bans. With the server database open,
use a prepared SQLite statement to insert `photon_ban(player, expires)`,
with a numeric account ID and either a UNIX expiry or NULL for permanent.
Delete the row to unban. There is no public ban administration route.

A valid Photon ticket is bearer material. This baseline uses a 15-minute
expiry and requires a live reservation for joins. Keep the deployment
private until the actual client's encrypted auth path is verified. Do not
enable protocol payload dumps. The integration test scans Luxon logs for the
issued tickets and local gate key. The probe uses explicit parameter 216
and is no evidence that the unmodified 20230414 flow populates this field.

## Photon Voice

The pinned RecFlare guide separates 20230414 Photon Voice from Photon
Realtime. Voice needs the client Voice connection and a compatible Voice
transport, including the codec and room behavior the build expects. This
extension is Realtime-only. Configurable Voice application IDs are retained,
but voice connection, routing and audio are unverified. No Tachyon path
was added.

## Memory

Release snapshots from the Debian 13 test host, measured as process RSS in
KiB. The last row adds the independent RSS values, so shared pages can be
counted twice. These samples exclude the TLS proxy and game clients.

| Run | KromaRec | Luxon | Sum |
| --- | ---: | ---: | ---: |
| musl KromaRec, two wire peers admitted | 3,628 | 3,744 | 7,372 |
| glibc KromaRec, two wire peers admitted | 23,760 | 3,800 | 27,560 |

With the standalone regression suite, the musl binary was 3,394,976 bytes,
using 3,324 KiB idle and 6,064 KiB with 100 HTTP keep-alives. The glibc binary
was 3,264,168 bytes, using 4,060 KiB idle and 25,684 KiB warm. These are
single-run snapshots, not peaks or sustained gameplay measurements.

## Remaining prerequisite

Run the [two-client checklist](multiplayer.md). The first unobserved stage
is the local-password flow in an actual 20230414 client. The next is the
client's ticket serialization and encryption at Photon authentication.
Then verify rendering of the embedded dorm and a supported room scene,
both directions of movement and gestures, and leave/rejoin. The transport
test intentionally uses test events in place of any undocumented game RPC.
