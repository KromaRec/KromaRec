# KromaRec 0.9.0

- Password changes now revoke previous HTTP access tokens, unused refresh
  tokens and existing Photon tickets. The password update and revocation
  commit in the same SQLite transaction.
- RecNet logout revokes previously issued Photon tickets. It keeps the
  early Orientation logout behavior and ignores logout with a superseded
  login lock. A transport disconnect alone still permits rejoining with a
  live reservation and ticket.
- Patched Luxon 1.5 checks ticket revocation through the existing private
  gate. Its wire format and build remain compatible with 0.8. Tickets
  issued by 0.8 need to be refreshed after upgrading.
- SQLite migration 7 keeps existing accounts, settings and rooms. Requests
  cancelled while SQLite work is active now hold the database permit until
  the blocking operation finishes.
- Regressions cover concurrent refresh replay, password and logout
  revocation, in-flight Photon handoffs, region discovery, disconnect and
  rejoin. The 689-route inventory retains its pinned source rows.
- Native 20230414 session acceptance remains pending. See
  [verification](compat/verification-0.9.md) and the
  [two-client guide](compat/multiplayer.md).
