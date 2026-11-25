# Native client acceptance checkpoint

| Status | Environment | Last evidenced stage |
|---|---|---|
| PASS | Windows developer handoff, 25 November 2026, backend 0.10.3 | Patch 0.3.1 loaded in BepInEx and intercepted discovery. Python reached the HTTPS directory with Caddy's public CA. |
| FAIL | Same historical native Windows launch | Discovery time check returned true, request-specific certificate check returned false, then TlsFatalAlert. No HTTP response status exists for that failed game request. |
| PASS | Linux 0.10.4 protocol tests | Authenticated accounts, scene metadata, allocation, secure gate, encrypted synthetic NameServer auth and ten-peer event delivery. |
| BLOCKED | Two native Windows 20230414 clients against 0.10.4 | No authorized game runtime supplied here. Native trust, login, scene rendering, shared movement, interactions, leave/rejoin and gameplay remain unverified. |
| NOT TESTED | Quest / PCVR / Photon Voice | No client trace, voice integration or Quest platform-auth evidence. |

1. On the Windows test PC, verify the authorized build and patch versions, LAN DNS, endpoint hostnames, issuing chain, validity and local clock. The historical build ID was unverified. Run the preflight against the installation with its public CA. Record Windows trust, Python and game outcomes separately.
2. Launch normally with original security checks. Record timestamp and sanitized TLS stage results. Investigate which supported trust source the game uses. The observed certificate failure does not establish which certificate or policy was rejected. Use a legitimately trusted deployment chain where available. Preserve signature, certificate, device and platform verification.
3. After verified discovery, record the first failing hostname, method, path template, status and endpoint-specific contract. Omit query values, account identifiers, tokens, cookies and gate secrets. For the historical Orientation 503, compare direct and proxied public room 13 on the deployment. The backend test covers metadata, private dorm allocation and transport admission.
4. Use two independent accounts, follow multiplayer.md, verify matching Photon room, instance and scene, then record movement and interaction in both directions on screen. Repeat with one player leaving and rejoining while the other stays.

The reference device-ID route receives oldDeviceId, newDeviceId and platform before account creation. Upstream documents a hang after its empty array response. KromaRec leaves the contract unresolved. Device reports, platform identity and local-password grants are separate flows. The backend's shared accounts, room allocation and Photon admission rely on authenticated account IDs. Local grants accept the configured 20230414 build policy. Steam and Meta identity validation and Quest-specific version, device and asset contracts require independent evidence.
