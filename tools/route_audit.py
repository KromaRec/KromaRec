"""Probe the pinned inventory against a new local database and render two inventories."""
import argparse
import csv
import http.client
import json
import os
import re
import secrets
import socket
import sys
import tempfile
import time
from collections import Counter
from pathlib import Path

from https_proxy_smoke import listen_port, running
from smoke import ROOT
from service_hosts_test import JSON_ROOTS, TEXT_ROOTS

PIN = "86bdd02063629677f120916d5d419440e0aeee71"
STATES = {"implemented but awaiting client verification": "IMPLEMENTED",
          "partially implemented": "PARTIALLY IMPLEMENTED", "not yet implemented": "MISSING"}


def example(path):
    path = path.replace("{[0-9a-fA-F-]{36}}", "{uuid}")
    def param(match):
        pattern = match.group(1)
        if pattern == "uuid":
            return "00000000-0000-0000-0000-000000000001"
        if pattern == "currency|tokens":
            return "tokens"
        return "1"
    return re.sub(r":[\w]+(?:\{([^{}]+)\})?", param, path).replace("*", "sample")


def read_rows(path):
    with path.open() as src:
        return list(csv.DictReader(src, delimiter="\t"))


def check_sample(service, method, path, status, body):
    if method == "GET" and path == "/":
        if service == "ns":
            directory = json.loads(body)
            assert status == 200 and len(directory) == len(set(directory.values())) == 33
            assert directory["Matchmaking"] == "https://match.rec.test"
        elif service in JSON_ROOTS:
            assert status == 200 and json.loads(body) == {"service":service, "status":"ok"}
        elif service in TEXT_ROOTS:
            assert status == 200 and body == b"hello, world!"
        else:
            return "NOT TESTED"
    elif (service, method, path) in {
        ("accounts","GET","/account/me"), ("playersettings","GET","/playersettings"),
        ("match","GET","/player/connection-info"), ("ai","GET","/gameai/user/access")
    }:
        assert (status, body) == (401, b"") or (service == "ai" and status == 401)
    elif (service, method, path) == ("auth","POST","/connect/token"):
        assert (status, body) == (415, b"")
    elif service == "accounts" and path == "/account/bulk" and method in ("GET","POST"):
        assert (status, body) == (200, b"[]")
    elif service == "match" and method == "PUT" and path in ("/player/photonregionpings", "/player/gameserverregionpings"):
        assert (status, body) == (200, b"")
    else:
        return "NOT TESTED"
    return "PASS"


def inspect(binary, reference):
    directory = ROOT / "compat"
    reference_rows = read_rows(directory / "upstream-routes.tsv")
    local = read_rows(directory / "route-status.tsv")
    assert len(local) == len(reference_rows) == 689
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        port = listen_port()
        env = {k:v for k,v in os.environ.items() if not k.startswith("KROMAREC_")}
        env.update(KROMAREC_DB=str(work/"test.db"), KROMAREC_JWT_SECRET=secrets.token_hex(32),
                   KROMAREC_DOMAIN="rec.test", KROMAREC_BIND=f"127.0.0.1:{port}",
                   KROMAREC_ASSETS=str(ROOT/"assets"))
        with running([binary], env, work/"server.log") as proc:
            for _ in range(100):
                if proc.poll() is not None:
                    raise RuntimeError("backend startup failed")
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=.2):
                        break
                except OSError:
                    time.sleep(.05)
            for ref, row in zip(reference_rows, local, strict=True):
                assert all(row[k] == v for k,v in ref.items())
                if reference:
                    lines = (reference/ref["source"]).read_text().splitlines()
                    pos = int(ref["line"]) - 1
                    assert 0 <= pos < len(lines)
                    assert ref["path"] in "\n".join(lines[pos:pos+12]), ref
                service, method, path = (row[k] for k in ("service", "method", "path"))
                state = STATES[row["status"]]
                result = {**ref, "implementation":state, "note":row["note"],
                          "transport_test":"NOT TESTED", "contract_test":"NOT TESTED",
                          "native_test":"NOT TESTED"}
                if method == "MOUNT":
                    result.update(implementation="NOT VERIFIED", note="Upstream router composition, no HTTP verb.")
                elif service == "datacollection":
                    result.update(implementation="INTENTIONALLY UNSUPPORTED", note="Telemetry origin excluded.")
                else:
                    url = example(path)
                    if method == "ALL":
                        url, method = "/audit-sample", "GET"
                    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=4)
                    try:
                        conn.request(method, url, headers={"Host":service+".rec.test"})
                        response = conn.getresponse()
                        body = response.read()
                        result.update(probe_method=method, probe_path=url, http=response.status,
                                      content_type=response.getheader("Content-Type"), transport_test="PASS")
                        if response.status >= 400:
                            result["error_body"] = body.decode("utf8")
                        result["contract_test"] = check_sample(service, method, path, response.status, body)
                    finally:
                        conn.close()
                yield result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--reference", type=Path, help="optional checkout at the documented commit")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.reference:
        import subprocess
        ref = subprocess.check_output(["git","-C",args.reference,"rev-parse","HEAD"], text=True).strip()
        assert ref == PIN
    rows = list(inspect(args.binary.resolve(), args.reference))
    args.output.mkdir(parents=True, exist_ok=True)
    base = args.output
    (base / "routes-0.10.4.json").write_text(json.dumps({"reference":PIN,"routes":rows},indent=2)+"\n")
    with (base / "routes-0.10.4.md").open("w") as doc:
        print("# KromaRec 0.10.4 reference inventory\n", file=doc)
        print("One row per pinned source registration, including repeated mounts. Implementation is the maintained code inventory. HTTP is one anonymous, empty-body local request on a fresh schema 9 database. A received HTTP response completes the transport probe. Contract PASS covers the specific asserted cases only. Other suites exercise additional routes without individual contract attribution here. Native acceptance is unverified throughout. Source lines, probe inputs, MIME types and exact error bodies are in the adjacent JSON.\n",file=doc)
        print("| Service | Method and template | Implementation | HTTP | Contract test |\n|---|---|---|---:|---|",file=doc)
        for r in rows:
            path=r["path"].replace("|",r"\|")
            print(f'| {r["service"]} | `{r["method"]} {path}` | {r["implementation"]} | {r.get("http","—")} | {r["contract_test"]} |',file=doc)
    print(dict(Counter(r["implementation"] for r in rows)))
    print(dict(Counter(r.get("http","skipped") for r in rows)))
    if any(r.get("http",0) >= 500 for r in rows):
        sys.exit("unexpected HTTP 5xx; inspect recorded rows")


if __name__ == "__main__":
    main()
