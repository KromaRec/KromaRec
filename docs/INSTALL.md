# Install KromaRec 0.10.3 on x86-64 Linux

Extract the ZIP into a fresh staging directory and run `sha256sum -c SHA256SUMS` inside the extracted package. Use `bin/kromarec` on Alpine and `bin/kromarec-x86_64-linux-gnu` on compatible glibc systems. The compiled Rust backend includes the routing changes. SQLite uses schema 9.

For a fresh manual installation as the existing server user on Alpine:

```sh
doas install -m 0755 bin/kromarec /usr/local/bin/kromarec
doas mkdir -p /usr/local/share/kromarec /var/lib/kromarec
doas cp -R assets /usr/local/share/kromarec/
doas chown "$(id -un):$(id -gn)" /var/lib/kromarec
```

Copy the example to a new private file only for initial setup. Generate two independent secrets once and store them without displaying them. Use the same Photon gate key in Luxon's private environment. Keep shell tracing off.

```sh
(
  set -eu
  set +x
  umask 077
  set -C
  {
    sed '/^KROMAREC_JWT_SECRET=/d; /^KROMAREC_PHOTON_GATE_KEY=/d' config/server.env.example
    printf 'KROMAREC_JWT_SECRET=%s\n' "$(openssl rand -hex 32)"
    printf 'KROMAREC_PHOTON_GATE_KEY=%s\n' "$(openssl rand -hex 32)"
  } > server.env
)
```

Set the app ID and region to match both clients. Run as the unprivileged service user with the existing environment loaded:

```sh
( set -eu; set -a; . ./server.env; set +a; exec /usr/local/bin/kromarec )
```

Configure the apex and wildcard DNS on the LAN. Adapt the suffix in `packaging/Caddyfile.example`, preserve Host and use your trusted TLS configuration. Only the HTTP backend is proxied. Trust the issuing public CA certificate on each client. The client service directory is `https://ns.rec.test/`.

For OpenRC use the service-specific paths, service account and init scripts in the README. A manual installation uses /usr/local/bin; the shipped OpenRC scripts use /usr/local/libexec. Use one startup method. For multiplayer use patched Luxon 1.5, configured from `packaging/luxon.config.example.yml`. Advertise the client-reachable server address on UDP 5058, 5055 and 5056. The bundled, tested Luxon executable is glibc. On Alpine build the patched source with `sh packaging/luxon/build.sh /path/to/a/fresh/build-dir` with Git, a C++ toolchain and CMake installed. See `compat/multiplayer.md` for launch and acceptance steps.

Verify the installed backend with `curl -sS -H 'Host: rooms.rec.test' http://127.0.0.1:8080/` and check with netstat that 8080 and 8081 are bound to 127.0.0.1. Run the Windows host sweep in API-AUDIT.md.
