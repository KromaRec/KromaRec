"""Compare every advertised service over direct HTTP and CA-verified Caddy HTTPS."""
import argparse
import http.client
import json
import os
from pathlib import Path
import secrets
import socket
import ssl
import subprocess
import tempfile
import time
from unittest.mock import patch

from https_proxy_smoke import listen_port, running
from smoke import ROOT

JSON_ROOTS = set("accounts ai cdn chat commerce discovery img playersettings roomcomments rooms".split())
TEXT_ROOTS = set("cards leaderboard link lists moderation platformnotifications storage studio".split())
# Public (anonymous) probes. IDs and POSTs are local to a disposable database.
PROBES = {
    "api": ("GET", "/api/versioncheck/v4?v=20230414", 200),
    "accounts": ("GET", "/account/me", 401),
    "auth": ("POST", "/connect/token", 415),
    "cdn": ("GET", "/config/LoadingScreenTipData", 200),
    "moderation": ("GET", "/voice/config", 200),
    "playersettings": ("GET", "/playersettings", 401),
    "rooms": ("GET", "/rooms/13", 200),
    "match": ("GET", "/player/connection-info", 401),
    "notify": ("POST", "/hub/v1/negotiate?negotiateVersion=1", 200),
}
for host, (path, status) in {
    "ai": ("/gameai/user/access", 401), "cards": ("/iam/me/channels/Announcements", 401),
    "chat": ("/thread/1/message", 404), "clubs": ("/club/1", 404),
    "commerce": ("/purchase/v1/hasspentmoney", 200), "discovery": ("/sections/bulk", 200),
    "econ": ("/api/influencerpartnerprogram/influencers", 404), "img": ("/openapi.json", 404),
    "leaderboard": ("/openapi.json", 404), "lists": ("/curatedlists", 200),
    "platformnotifications": ("/preferences", 404), "roomcomments": ("/comments/get/1", 200),
    "storage": ("/openapi.json", 404), "studio": ("/cloud-builds", 400), "www": ("/privacy", 404),
}.items():
    PROBES[host] = ("GET", path, status)
PROBES["link"] = ("POST", "/actionlink", 404)


def request(conn, host, method, path):
    try:
        conn.request(method, path, headers={"Host": host})
        r = conn.getresponse()
        return r.status, r.getheader("Content-Type", ""), r.read()
    finally:
        conn.close()


def test(binary, caddy, baseline):
    rows = []
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        ports = set()
        while len(ports) < 3:
            ports.add(listen_port())
        backend_port, tls, gate = ports
        env = {**os.environ, "KROMAREC_BIND": f"127.0.0.1:{backend_port}", "KROMAREC_DOMAIN": "rec.test",
               "KROMAREC_SUBDOMAINS": "{}", "KROMAREC_DB": str(work / "test.sqlite3"),
               "KROMAREC_ASSETS": str(ROOT / "assets"), "KROMAREC_JWT_SECRET": secrets.token_hex(32),
               "KROMAREC_PHOTON_REALTIME_APP_ID": "integration-test-app",
               "KROMAREC_PHOTON_GATE_ADDR": f"127.0.0.1:{gate}", "KROMAREC_PHOTON_GATE_KEY": secrets.token_hex(32),
               "XDG_DATA_HOME": str(work / "data"), "XDG_CONFIG_HOME": str(work / "config")}
        config = (ROOT / "packaging/Caddyfile.example").read_text().replace("8080", str(backend_port)).replace("rec.example.test", "rec.test")
        cf = work / "Caddyfile"
        cf.write_text(f"{{\nadmin off\nhttps_port {tls}\nauto_https disable_redirects\nskip_install_trust\n}}\n" + config)
        ca = work / "data/caddy/pki/authorities/local/root.crt"
        with (running([binary], env, work / "backend.log") as backend,
              running([caddy, "run", "--config", cf], env, work / "proxy.log") as proxy):
            for _ in range(150):
                if backend.poll() is not None or proxy.poll() is not None:
                    raise RuntimeError((work / "backend.log").read_text() + (work / "proxy.log").read_text())
                if ca.exists():
                    try:
                        with socket.create_connection(("127.0.0.1", tls), timeout=.2):
                            break
                    except OSError:
                        pass
                time.sleep(.1)
            else:
                raise RuntimeError("Caddy startup timed out")
            ctx = ssl.create_default_context(cafile=ca)
            def direct(host, method, path):
                return request(http.client.HTTPConnection("127.0.0.1", backend_port, timeout=5), host, method, path)
            def https(host, method, path):
                conn = http.client.HTTPSConnection(host, tls, context=ctx, timeout=5)
                # Only change the TCP destination. Preserve the hostname for SNI and certificate verification.
                with patch.object(conn, "_create_connection", lambda *a, **kw: socket.create_connection(("127.0.0.1", tls), timeout=5)):
                    return request(conn, host, method, path)
            directory = json.loads(https("ns.rec.test", "GET", "/")[2])
            expected = json.loads((ROOT / "compat/service-hosts.json").read_text())
            assert directory == {k: f"https://{s}.rec.test" for k,s in expected.items()
                                 if k not in {"BugReporting", "GameLogs", "DataCollection"}}
            assert len(directory) == 33
            for label, url in sorted(directory.items()):
                host = url.removeprefix("https://")
                svc = expected[label]
                try:
                    socket.getaddrinfo(host, 443)
                    dns = "resolved"
                except OSError:
                    dns = "unresolved"
                r = https(host, "GET", "/")
                assert r == direct(host, "GET", "/"), host
                if not baseline:
                    if svc in JSON_ROOTS:
                        assert r[0] == 200 and r[1].startswith("application/json") and json.loads(r[2]) == {"service":svc,"status":"ok"}, host
                    elif svc in TEXT_ROOTS:
                        assert r[0] == 200 and r[1].startswith("text/plain") and r[2] == b"hello, world!", host
                    else:
                        assert r[0] == (401 if svc == "match" else 404), host
                assert https(host, "GET", "/__kromarec_missing__")[0] in (401, 404), host
                probe = PROBES.get(svc)
                result = "No non-root route in pinned inventory" if not probe else ""
                if probe:
                    method, path, status = probe
                    a, b = direct(host, method, path), https(host, method, path)
                    assert a[:2] == b[:2], (host, a[:2], b[:2])
                    if svc == "notify":
                        assert json.loads(b[2])["negotiateVersion"] == 1
                    else:
                        assert a[2] == b[2], host
                    if not baseline:
                        assert b[0] == status, (host, b[0], status)
                    result = f"{method} {path} → {b[0]}"
                rows.append([label, host, dns, r[0], r[1], r[2].decode(), result])
            with socket.create_connection(("127.0.0.1",gate),timeout=2):
                pass
        if not baseline:
            for overrides in ({"cdn":"cms"}, {"www":"NS"}, {"strings":"Geo"}):
                collision = subprocess.run([binary], env={**env, "KROMAREC_SUBDOMAINS":json.dumps(overrides)},
                                           capture_output=True, timeout=5)
                assert collision.returncode and b"same subdomain" in collision.stderr
        bad_gate_env = {**env, "KROMAREC_PHOTON_GATE_ADDR": f"0.0.0.0:{gate}"}
        denied = subprocess.run([binary], env=bad_gate_env, capture_output=True, timeout=5)
        assert denied.returncode != 0 and b"gate must bind to 127.0.0.1" in denied.stderr
    return rows


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--caddy", default="caddy")
    parser.add_argument("--baseline", action="store_true", help="record earlier release responses without enforcing new statuses")
    args = parser.parse_args()
    print(json.dumps(test(args.binary.resolve(), args.caddy, args.baseline), indent=2))
