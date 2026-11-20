# KromaRec 0.8.0

- Process the SignalR JSON handshake and later records when they share one
  WebSocket message. The reconnect regression checks that ping, room updates,
  presence and invitations still work after opening a new socket.
- Add opt-in startup diagnostics with fixed labels and numeric statuses. The
  trace omits arbitrary request data, including URLs, IDs, headers and bodies.
- Add a client-side HTTPS preflight for trusted discovery, MatchUrl, version,
  game config and a CDN bootstrap resource.
- Document the native acceptance procedure for the Phase 1 Windows patch and
  the pinned Steam manifest. Keep the 689-entry compatibility inventory.
- Keep SQLite schema 6, authentication DTOs, Photon ticket and admission
  contracts, and the existing patched Luxon 1.5 binary.
