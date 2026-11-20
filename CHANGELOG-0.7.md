# KromaRec 0.7.0

- Added a source overlay for the pinned RecFlare BepInEx patch. It receives
  Realtime tickets from the configured HTTPS match service and supplies them
  to matching encrypted Photon NameServer authentication. The 20230414
  interop build and runtime behavior still need real-client verification.
- Removed credential-bearing HTTP bodies and query strings from the overlay's
  diagnostics. Added `Cache-Control: no-store` to connection information.
- Reused a prepared SQLite lookup for live Photon reservation checks instead
  of building a complete room DTO during authentication and admission.
- Preserved a newer login and reservation when a stale logout supplies its
  old login lock. Legacy logout without a lock and Orientation behavior are
  retained.
- Extended regression coverage for AuthenticateOnce, stale logout, ticket
  caching headers, and data continuity from 0.6. The SQLite schema stays at
  6 and the 689-entry compatibility inventory remains intact.
- Added example client settings and reproducible RSS measurements. The
  existing OpenRC, Caddy and patched Luxon setup remains supported.
