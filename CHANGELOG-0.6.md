# KromaRec 0.6.0

- Added signed, audience-scoped Realtime JWTs with 15-minute expiry and
  a distinct token purpose. The existing RecNet verifier still rejects
  Photon credentials.
- Added a bounded, keyed loopback authorization listener and a pinned
  Luxon 1.5 transport extension. Luxon derives player identity from the
  signed account ID, rechecks it during token handoffs, and checks the
  active RecNet reservation on Master and Game admission.
- Added permanent and expiring Photon bans in SQLite migration 006.
  New matchmakes and transport admissions consult the ban table.
- Added authenticated exclusive-login and verification handlers backed by
  the existing lobby login lock. The upstream reference currently just
  acknowledges these calls. KromaRec returns 409 for a stale or missing lock.
- Extended the 0.5 suite to cover denied transport joins, failed handoffs,
  identity spoofing, expiry, room reservation revocation, concurrent joins,
  migration, SignalR and existing room behavior.
- Added build, Caddy, OpenRC and two-client acceptance instructions.
  Kept the pinned 689-route inventory. No actual game-client result yet.
