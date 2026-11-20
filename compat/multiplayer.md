# KromaRec 0.10.1 two-client deployment and acceptance

## Prepare the server

1. Use the bundled KromaRec static musl executable on Alpine x86-64 or the
   bundled glibc executable on Debian 13 or a compatible distribution.
   Back up an existing database before starting 0.10. Schema 8 preserves
   existing room coordinates and adds a default matchmaking policy.
2. Run the bundled `bin/luxon-kromarec-x86_64-linux-gnu` on a compatible
   glibc host. On Alpine, build the patched transport natively with Git,
   CMake 3.21+, a sufficiently recent C++23 compiler and standard library,
   make, Python 3 and development headers. Run
   `packaging/luxon/build.sh /path/to/fresh-build`. It checks out the
   exact [Luxon 1.5 commit](https://github.com/niansa/LuxonServer/tree/e4bab98d8d6b3aee9ecf4035309a78a59b584f93)
   and builds `build/luxon_server`. The supported integration is same-host.
3. Set up DNS for your RecNet suffix and every advertised subdomain.
   Terminate HTTPS at a local proxy. See `packaging/Caddyfile.example`.
   Trust the proxy's CA on both clients. Preserve the original Host header
   and enable WebSocket upgrades on the notify host. Keep the gate off the
   proxy. Keep request/response logging disabled on authentication routes.
4. Export the same gate address and randomly generated gate key to KromaRec
   and patched Luxon. Only KromaRec needs the JWT secret. Place secrets in
   service configuration readable by the service operator only.

```sh
export KROMAREC_PHOTON_GATE_ADDR=127.0.0.1:8081
export KROMAREC_PHOTON_GATE_KEY="$(openssl rand -hex 32)"
export KROMAREC_JWT_SECRET="$(openssl rand -hex 32)"
export KROMAREC_DOMAIN=rec.example.test
export KROMAREC_BIND=127.0.0.1:8080
export KROMAREC_PHOTON_REALTIME_APP_ID=7a6eafd9-8a49-47bd-98da-c743525a3e72
export KROMAREC_PHOTON_REGION=us
```

Preserve the secrets across service restarts. Choose a local UUID for Realtime
and put the same value in the client configuration. No hosted Photon app is
needed for this self-hosted setup.

5. Copy `packaging/luxon.config.example.yml` to `config.yml` in Luxon's
   working directory. Advertise the server's client-reachable IP on UDP
   ports **5058** (Name), **5055** (Master), and **5056** (Game). Keep the
   region `us` consistent in both configs. Permit UDP only from your
   private test network. Use a trusted VPN when crossing untrusted networks.
   The wire harness intentionally uses plaintext Photon parameters in
   loopback tests. Encryption behavior of the game client remains untested.
6. Start KromaRec first, then patched Luxon. For a direct launch, run
   `./bin/kromarec` from the project directory with the environment above
   and `KROMAREC_DB` set to a persistent path. In a second terminal, export
   the same gate settings, change to the directory containing Luxon's
   `config.yml`, and run the patched executable. Stop Luxon first, then
   stop KromaRec with SIGTERM. With OpenRC, install the core
   as described in the main README, then install the transport as follows.

```sh
install -m 0755 /path/to/build/luxon_server /usr/local/libexec/luxon-kromarec
install -d -m 0755 /etc/kromarec/luxon
install -m 0644 packaging/luxon.config.example.yml /etc/kromarec/luxon/config.yml
install -m 0755 packaging/luxon.initd /etc/init.d/luxon
install -m 0600 packaging/luxon.confd /etc/conf.d/luxon
# Edit the advertised addresses and both service configs before starting.
rc-service luxon start
rc-update add luxon default
# Shutdown in dependency order.
rc-service luxon stop
rc-service kromarec stop
```

## Prepare the clients

1. Use two Windows x64 **20230414** PC installations. Steam app is
   `471710`, depot `471711`, manifest `6426603215211043630`. Obtain the
   depot using an account that has access. Install BepInEx 6.0.0-pre.2
   IL2CPP and start the game once to generate `BepInEx/interop`.
2. Use the version-matched **KromaRec-Patch 0.3.x** build instructions and
   generated interop for this exact game installation. Install one redirect
   plugin. Keep the original device, anti-cheat, integrity and certificate
   checks active. The older source overlay in this backend is historical.
   The backend-facing contract is HTTPS discovery, manual local-account
   authentication, and a fresh signed ticket from the configured Match
   origin at `/player/connection-info`. Photon Authenticate or AuthenticateOnce
   must carry the ticket as `token=<ticket>` in custom authentication
   parameter 216. Only the game can confirm the binding and encryption.
3. On each client, configure `net.kromarec.patch.cfg` with your actual
   `Backend.BaseUrl`, exact `Backend.MatchUrl`, and the same Realtime
   UUID, Photon NameServer address, UDP port, and region as the server.
   Install the HTTPS CA in the client trust store. Run the preflight from
   the client-side machine before launching the game:

   ```sh
   python3 tools/https_preflight.py https://ns.rec.example.test --match-url https://match.rec.example.test
   ```

   The checker validates certificate trust, discovery, MatchUrl, build
   selection, public bootstrap resources, Orientation metadata and SignalR
   negotiation without using account credentials. Add `--ca path/to/ca.pem` if Python needs the local CA
   explicitly. A successful preflight does not verify Unity's TLS path.
4. Enable local signup temporarily and provision two independent
   accounts with form `grant_type=create_account`, `password` and
   `ver=20230414` on `auth POST /connect/token`. Use each issued bearer
   with `accounts GET /account/me` to read its generated username.
   Keep passwords and tokens out of shell history and logs. Disable
   signup after provisioning. Use manual login once per installation.
5. For the first capture, set `KROMAREC_TRACE_STARTUP=true` in the backend
   environment and restart the backend. Use credential-redacted client diagnostics
   to correlate the login, scene and Photon stages. Disable tracing after the
   run. Verify a fresh connection-info response followed by successful
   encrypted NameServer auth. If those stages do not occur, record the
   first failed HTTP status or Photon return code and which hook bound.
   A media or scene failure after Game admission is a separate stage.
   The device-ID acknowledgement remains pending. If the original device
   check blocks account creation or login, record the route, HTTP status,
   build, and the last completed screen. Redact device IDs. A fabricated
   success response is known to hang the reference flow.

## Run the acceptance test

| Step | Pass evidence |
| --- | --- |
| Both sign in and reach their dorms | Independent profiles, startup calls succeed, each client renders its own dorm. New accounts may first enter Orientation. |
| A creates a room, publishes it and enters | A built-in scene such as MakerRoom renders. A's room lookup and matchmake return the expected scene key. |
| B discovers and joins | Both matchmake responses agree on instance ID, room name, scene ID and region. Luxon authenticates both and assigns distinct actors in one Game room. |
| Both interact | Show movement in both directions and gestures or interactions on the other client's screen. Capture timestamps and which client moved. |
| B leaves and rejoins | A stays in the instance. B re-enters and both directions of movement still replicate. |
| Party invite | With B in A’s party, A starts room navigation. B receives the legacy invite and can join the same instance. Compare the returned instance ID and Photon room on both clients. |
| Private-room check | A invites B. An authenticated account without a reservation fails to join by the Photon room name. |

If a step fails, record the first failed host/route or Photon hop and return
code. For scene failures, also record the Unity scene ID and missing asset
path. Keep passwords, Authorization headers, and Photon tickets out of traces.
Built-in room DTOs and their embedded scene IDs are shipped. Proprietary game
assets and arbitrary uploaded room saves need a compatible client or
operator-provided asset data.

## Session behavior

Photon tickets expire after 15 minutes. After a completed RecNet logout,
request a fresh ticket and allocate or rejoin the room through RecNet. During
the existing Orientation startup sequence, logout preserves the reservation
and ticket. A password change requires fresh HTTP credentials and a fresh
Photon ticket. A direct Photon transport disconnect preserves the reservation
until RecNet logout or expiry.
Use the Luxon extension built or bundled with **0.10.1**. It holds one
loopback connection per Game actor. Logout and password changes invalidate
open SignalR sockets and notify Luxon after the database commit. Luxon
disconnects the actor on its existing slow update. An account can reconnect
to SignalR with a still-valid HTTP bearer after normal RecNet logout.
Password changes require a new bearer. SignalR also closes at bearer expiry.
The Photon ban table applies to Realtime only.

Reservation loss, room deletion and operator SQL bans are checked every five
seconds while the Game actor is connected. A stalled backend stops renewing
the lease; Luxon checks its 12-second deadline on its next slow update. An
admission ticket must be current at each new handoff and join. Already
admitted actors use their live reservation and lease. Allow up to 128 live
Game leases and 16 concurrent gate handshakes per KromaRec process. Run one
KromaRec process against its database on the same host as Luxon.

With tracing enabled the gate emits fixed stage names for Name, Master,
Game auth and admission. Matchmaking emits a numeric result and a recognized
built-in scene ID, `other` or `none`. CDN diagnostics report a status and
route class. Raw URLs, asset paths, IDs of accounts and tokens are omitted.

## Repeat the automated test

```sh
python3 tools/smoke.py bin/kromarec
python3 tools/rooms_smoke.py bin/kromarec --secure-luxon bin/luxon-kromarec-x86_64-linux-gnu
# Add --baseline /path/to/kromarec-0.9 to exercise migration.
python3 tools/https_proxy_smoke.py bin/kromarec
```

For a repeatable HTTP RSS measurement run `python3 tools/memory_rss.py bin/kromarec bin/kromarec-x86_64-linux-gnu`. The secure room suite prints separate and combined RSS when both Photon peers are admitted.

The suite starts temporary local processes and sends real Photon operations.
It tests transport admission and event relay. See [verification](verification-0.10.1.md)
for its coverage and the outstanding gameplay and Photon Voice checks.


The HTTPS smoke test exercises the shipped Caddy routing, an isolated CA and
an authenticated SignalR WebSocket upgrade. It uses a synthetic client. On a
booted Alpine host, verify both OpenRC services report running after startup,
run the client-side HTTPS preflight, then stop Luxon before KromaRec and verify
both services report stopped. Namespaced binary execution does not establish
OpenRC service lifecycle behavior.
