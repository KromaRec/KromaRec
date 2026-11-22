# KromaRec 0.10.1 routing update

Added 13 pinned public service health routes, routed overridden discovery and service names, and honored the www override in the API share URL. The release includes per-host HTTPS regression coverage and preserves schema 8. See [the audit](docs/API-AUDIT.md) for results and remaining gaps.

# KromaRec 0.5.0

- Match the pinned RecFlare startup flow by persisting new accounts in the canonical Orientation scene and retaining it during the client's early logout. Later room entry clears onboarding presence.
- Answer the platform picker for unlinked local accounts, and implement bounded GET and form POST room bulk lookup. Guests who redeemed a room invitation can fetch that room's scene metadata while present.
- Add a tested Luxon 1.5 sample configuration and a real ENet/Photon protocol probe. The probe consumes a KromaRec room allocation and tests transport handoff, bidirectional events, leave and rejoin.
- Apply SQLite migration 005 in place. Retain all 689 upstream compatibility entries.
- Document actual game-client acceptance steps, stock Luxon authentication limits and the unverified Photon Voice path.

See [verification](compat/verification-0.5.md) for measured results.

# KromaRec 0.4.0

## Added

- SQLite migration 004 for rooms, room roles, room instances, live player locations and invitations. Existing account, refresh, relationship, inbox and settings records are migrated in place.
- Built-in RecCenter and template scenes from the pinned RecFlare release, plus private player dorms.
- Persistent room creation, template cloning, search, metadata editing, deletion, co-ownership offers, role changes and ownership transfer.
- Transactional 2023 session allocation with a capacity limit, public reuse, private sessions, an owner instance listing, expiry and round-in-progress state.
- Configurable Photon Realtime, Voice and Chat app IDs, a region and signed Photon identity. Photon tokens cannot be used as RecNet bearer tokens.
- Live SignalR JSON WebSockets for room edits, game invites and redacted friend locations. Co-owner invitations also appear in the inbox.
- Local password grants now include RecFlare-style client JWT claims.
- Automated migration, room, session and notification coverage. The compatibility matrix retains each of the original 689 entries.

## Verification

The static musl and glibc builds passed the HTTP and SignalR suites. Two HTTP
players reached the same room instance and received the same Photon room name.
Luxon 1.5 was started separately and bound its UDP ports. Real client
authentication, the Photon handshake and in-game interaction remain
unverified. See the [test report](compat/verification-0.4.md) for memory
measurements and the [multiplayer runbook](compat/multiplayer.md) for setup.

# KromaRec 0.3.0

## Added

- Password changes using the authenticated upstream route. Changes verify the old password when set, use Argon2id, and revoke all persisted refresh tokens in the same transaction. Existing access tokens still expire normally. Local passwords retain the 8–128 byte limit.
- Persistent public bios, identity flags, and pronoun flags with authenticated writes.
- Transactional friend requests, crossed-request acceptance, accept/remove/direct add, mutual friends, and per-player ignore, mute, and favorite flags. Removes preserve flags.
- A durable inbox for friend-request and acceptance messages, bounded to 20 messages per recipient. Message deletion is scoped to the recipient.
- SQLite-backed lobby login, heartbeat and logout with expiring presence, stale login-lock handling on heartbeat, and the online-friend count.
- SQLite schema migration 003. Existing accounts, password hashes, tokens and settings are retained.
- HTTP tests for the new routes and their main error paths, with the original 689 upstream inventory rows retained.

## Compatibility limits

The lobby presence records do not include rooms or in-game instance state. Profile and social changes appear on the next HTTP read. Live SignalR notifications, direct chat, game invites, platform login, recovery, account deletion and abuse controls remain pending. No room lifecycle, matchmaking, Photon or Tachyon process was added. Economy, progression, leaderboards, moderation and asset uploads remain pending. Telemetry remains disabled. See the route matrix for per-declaration status.

## Verification and footprint

The full HTTP smoke suite passed with the x86-64 static musl and glibc release builds on the Linux build host. A database created by the 0.2 executable migrated to version 3 with its account and settings intact and passed SQLite integrity check. The authoritative upstream inventory still has 689 entries. The matrix records 63 as implemented and awaiting client verification, including 32 newly smoke-tested rows. The remaining 626 include incomplete, excluded telemetry, and unported entries.

The static musl build measured 2,800 KiB RSS at idle and 5,388 KiB after the expanded smoke workload while holding 100 HTTP keep-alive connections. The prior 0.2 review recorded 2,824 KiB and 5,424 KiB respectively. The difference is within run-to-run variation and the workload changed. The musl binary grew by 110,592 bytes. In the glibc run, RSS was 3,660 KiB at idle and 25,084 KiB after the same workload. That post-workload sample is an observed glibc footprint, without allocator attribution. These are process RSS snapshots, not game-session or peak-memory benchmarks. The actual client, live upstream comparison, and Alpine/OpenRC runtime remain unverified.
