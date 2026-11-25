"""Local regressions for configured startup, account reads and media types."""
import json
import os
from pathlib import Path
import subprocess
import tempfile


def startup(binary):
    assert subprocess.check_output([binary, "--version"], text=True).strip() == "KromaRec 0.10.4"
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "not-created.sqlite3"
        env = {k:v for k,v in os.environ.items() if not k.startswith("KROMAREC_")}
        result = subprocess.run([binary], env={**env, "KROMAREC_DB":str(path)},
                                capture_output=True, timeout=5)
        assert result.returncode and b"requires KROMAREC_JWT_SECRET" in result.stderr
        assert not path.exists()


def routes(request, account):
    for path in ("/player/photonregionpings", "/player/gameserverregionpings"):
        status, _, data = request("match", path, "PUT", '{"us":10}', **{"Content-Type":"application/json"})
        assert (status, data) == (200, b"")
        assert request("match", path)[0] == 401
    data = json.loads(request("accounts", "/account/bulk?id=2,2&id=99")[2])
    assert [x["accountId"] for x in data] == [2, 99]
    assert data[0]["username"] == account["username"] and "email" not in data[0]
    headers = {"Content-Type":"Application/X-Www-Form-Urlencoded; charset=UTF-8"}
    status, _, data = request("accounts", "/account/bulk?id=99", "POST", "id=2&id=99", **headers)
    assert status == 200 and [x["accountId"] for x in json.loads(data)] == [99, 2]
    assert request("accounts", "/account/bulk?id="+",".join(map(str, range(101))))[0] == 400
    assert request("auth", "/connect/token", "POST", "grant_type=password",
                   **{"Content-Type":"application/x-www-form-urlencoded-invalid"})[0] == 415
