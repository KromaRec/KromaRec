"""Reproduce the 0.10.3 configuration failure and benchmark a fresh local instance."""
import argparse
from contextlib import contextmanager
import http.client
import json
import os
from pathlib import Path
import secrets
import sqlite3
import statistics
import subprocess
import tempfile
import time

from https_proxy_smoke import listen_port


@contextmanager
def launch(binary, path, *, configured):
    port = listen_port()
    env = {k:v for k,v in os.environ.items() if not k.startswith("KROMAREC_")}
    env.update(KROMAREC_BIND=f"127.0.0.1:{port}", KROMAREC_DB=str(path))
    if configured:
        env["KROMAREC_JWT_SECRET"] = secrets.token_hex(32)
    start = time.perf_counter()
    with subprocess.Popen([binary], env=env, stderr=subprocess.PIPE) as proc:
        try:
            assert b"listening" in proc.stderr.readline()
            yield port, 1000 * (time.perf_counter() - start)
        finally:
            proc.terminate()
            proc.wait(timeout=10)


def fetch(port, host, path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    try:
        conn.request("GET", path, headers={"Host": host + ".rec.test",
                     "Accept": "text/html,application/json;q=0.9,*/*;q=0.8"})
        response = conn.getresponse()
        return response.status, response.read()
    finally:
        conn.close()


def measure(binary, directory):
    db = directory / "test.sqlite3"
    snapshot = directory / "snapshot.sqlite3"
    with launch(binary, db, configured=True) as (port, elapsed):
        print(f"{binary.name} fresh startup ms {elapsed:.3f}")
        for host, path in (("ns", "/"), ("rooms", "/rooms/13")):
            times = []
            for _ in range(100):
                start = time.perf_counter()
                assert fetch(port, host, path)[0] == 200
                times.append(1000 * (time.perf_counter() - start))
            print(f"{host} {path} median/p95 ms {statistics.median(times):.3f} {sorted(times)[94]:.3f}")
        with sqlite3.connect(db) as live, sqlite3.connect(snapshot) as backup:
            assert live.execute("pragma journal_mode").fetchone() == ("wal",)
            live.backup(backup)
    with launch(binary, snapshot, configured=True) as (port, _):
        assert json.loads(fetch(port, "rooms", "/rooms/13")[1])["RoomId"] == 13
        with sqlite3.connect(snapshot) as restored:
            assert restored.execute("pragma integrity_check").fetchone() == ("ok",)
            assert not restored.execute("pragma foreign_key_check").fetchall()
            assert restored.execute("pragma user_version").fetchone() == (9,)
        print("PASS | WAL snapshot reopened, schema and integrity")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("binary", type=Path)
    cli.add_argument("--baseline", type=Path, help="0.10.3 executable")
    args = cli.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        if args.baseline:
            db = work / "unopened.sqlite3"
            with launch(args.baseline.resolve(), db, configured=False) as (port, _):
                assert fetch(port, "ns", "/")[0] == 200
                assert fetch(port, "rooms", "/rooms/13") == (503, b"")
                assert not db.exists()
            print("PASS | reproduced 0.10.3 discovery 200, Orientation 503")
            (work / "old").mkdir()
            measure(args.baseline.resolve(), work / "old")
        measure(args.binary.resolve(), work)
