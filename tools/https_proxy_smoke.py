#!/usr/bin/env python3
"""Verify the shipped Caddy host routing and SignalR upgrade using a test CA."""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import socket
import ssl
import subprocess
import tempfile
import time
from unittest.mock import patch
from urllib.parse import urlencode
from urllib.request import HTTPSHandler, Request, build_opener

from tools.https_preflight import verify
from tools.rooms_smoke import HubSocket
from tools.smoke import ROOT, SECRET


def listen_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


@contextmanager
def running(command, env, log_path):
    with log_path.open("w+") as log:
        process = subprocess.Popen(command, env=env, stdout=log, stderr=log)
        try:
            yield process
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=12)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


def run(binary, caddy):
    with tempfile.TemporaryDirectory() as temp:
        work = Path(temp)
        backend_port, proxy_port = listen_port(), listen_port()
        while proxy_port == backend_port:
            proxy_port = listen_port()
        environment = {
            **os.environ, "KROMAREC_BIND": f"127.0.0.1:{backend_port}",
            "KROMAREC_DOMAIN": "rec.test", "KROMAREC_ASSETS": str(ROOT / "assets"),
            "KROMAREC_DB": str(work / "rec.sqlite3"), "KROMAREC_JWT_SECRET": SECRET,
            "KROMAREC_PASSWORD_SIGNUP": "true", "XDG_DATA_HOME": str(work / "data"),
            "XDG_CONFIG_HOME": str(work / "config"), "no_proxy": "*",
        }
        config = (ROOT / "packaging/Caddyfile.example").read_text()
        config = config.replace("rec.example.test", "rec.test").replace("8080", str(backend_port))
        config_file = work / "Caddyfile"
        config_file.write_text(
            f"{{\nadmin off\nhttps_port {proxy_port}\n"
            "auto_https disable_redirects\nskip_install_trust\n}\n" + config
        )
        ca = work / "data/caddy/pki/authorities/local/root.crt"
        resolve = socket.getaddrinfo

        def loopback(host, port, *args, **kwargs):
            if host == "rec.test" or host.endswith(".rec.test"):
                return resolve("127.0.0.1", proxy_port, *args, **kwargs)
            return resolve(host, port, *args, **kwargs)

        with (running([binary], environment, work / "backend.log") as backend,
              running([caddy, "run", "--config", config_file], environment, work / "caddy.log") as proxy,
              patch("socket.getaddrinfo", loopback), patch.dict(os.environ, {"no_proxy": "*"})):
            for _ in range(100):
                assert backend.poll() is None and proxy.poll() is None, "backend or Caddy exited"
                if ca.is_file():
                    try:
                        with socket.create_connection(("127.0.0.1", proxy_port), timeout=.1):
                            break
                    except OSError:
                        pass
                time.sleep(.1)
            else:
                raise RuntimeError("Caddy did not start or generate its test CA")
            try:
                verify("https://ns.rec.test", "https://match.rec.test")
            except ValueError as exc:
                assert str(exc) == "HTTPS discovery: URLError"
            else:
                raise AssertionError("the private test CA was accepted without explicit trust")
            verify("https://ns.rec.test", "https://match.rec.test", str(ca))

            context = ssl.create_default_context(cafile=str(ca))
            opener = build_opener(HTTPSHandler(context=context))
            form = urlencode({"grant_type": "create_account", "password": "proxy-smoke-test-password"})
            signup = Request("https://auth.rec.test/connect/token", data=form.encode())
            with opener.open(signup, timeout=5) as response:
                token = json.load(response)["access_token"]
            negotiate = Request("https://notify.rec.test/hub/v1/negotiate?negotiateVersion=1", method="POST")
            with opener.open(negotiate, timeout=5) as response:
                connection = json.load(response)["connectionToken"]
            hub = HubSocket(proxy_port, token, connection, batched=True, tls=context)
            try:
                hub.send({"type": 6})
                assert hub.receive() == {"type": 6}
            finally:
                hub.close()
            print("PASS | HTTPS discovery, Orientation metadata, local signup and SignalR WSS via Caddy")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--caddy", default="caddy")
    arguments = parser.parse_args()
    run(arguments.binary.resolve(), arguments.caddy)
