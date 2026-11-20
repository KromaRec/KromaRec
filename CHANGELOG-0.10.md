# KromaRec 0.10.0

- Room navigation accepts repeated `AdditionalPlayerIds` fields from the
  20230414 client. A successful allocation records up to 16 distinct party
  invitations and sends the legacy SignalR game-invite message. The invited
  account must still join through RecNet before Luxon grants admission.
- Added live instance privacy and matchmaking policy. Only room managers can
  mark an instance private. Members can set Default, Avoid or Ignore for
  automatic allocation. Established reservations continue to work.
- SQLite migration 008 gives existing instances the Default policy while
  preserving room coordinates, account data and active reservations.
- Added HTTPS startup checks for the Orientation scene ID and SignalR
  negotiation, plus an end-to-end Caddy HTTPS and WSS regression probe.
  Startup trace labels distinguish login, verification, heartbeat and
  logout without exposing credentials.
- Expanded protocol tests to cover party invitations, sender actor identity,
  instance admission and allocation. Added repeatable RSS and process CPU
  measurements. Verified native 20230414 gameplay remains a separate milestone.
