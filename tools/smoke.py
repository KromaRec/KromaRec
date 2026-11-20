#!/usr/bin/env python3
"""Smoke-test a built KromaRec binary with synthetic 20230414 HTTP requests."""
import argparse
import base64
import concurrent.futures
import hashlib
import hmac
import http.client
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import tempfile
import time
from urllib.parse import urlencode

SECRET = "test-only-key-kromarec-0123456789"
ROOT = Path(__file__).resolve().parents[1]


def signed_token(account, expiry, alg="HS256"):
    def encode(data):
        return base64.urlsafe_b64encode(data).rstrip(b"=")

    head = encode(json.dumps({"alg": alg}).encode())
    body = encode(json.dumps({"sub": account, "exp": expiry}).encode())
    signed = head + b"." + body
    signature = hmac.digest(SECRET.encode(), signed, "sha256")
    return (signed + b"." + encode(signature)).decode()


def rss(pid):
    with open(f"/proc/{pid}/smaps_rollup") as stats:
        return int(next(line.split()[1] for line in stats if line.startswith("Rss:")))


def smoke(binary, work, schema=8, observe=None):
    assets = work / "assets"
    shutil.copytree(ROOT / "assets", assets)
    (assets / "room").mkdir()
    content = bytes(range(256)) * 8192
    (assets / "room" / "blob").write_bytes(content)
    (work / "private").write_bytes(b"private")
    (assets / "room" / "outside").symlink_to(work / "private")
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    process = subprocess.Popen([binary], env={
        **os.environ,
        "KROMAREC_DOMAIN": "rec.test",
        "KROMAREC_SUBDOMAINS": '{"playersettings":"settings"}',
        "KROMAREC_BIND": f"127.0.0.1:{port}",
        "KROMAREC_ASSETS": str(assets),
        "KROMAREC_DB": str(work / "settings.sqlite3"),
        "KROMAREC_JWT_SECRET": SECRET,
        "KROMAREC_MAX_CONNECTIONS": "128",
        "KROMAREC_PASSWORD_SIGNUP": "true",
    }, stderr=subprocess.PIPE, text=True)

    def request(host, path, method="GET", body=None, **headers):
        conn = http.client.HTTPConnection("127.0.0.1", port, timeout=4)
        try:
            conn.request(method, path, body, {"Host": host + ".rec.test" if host else "rec.test", **headers})
            response = conn.getresponse()
            return response.status, response.headers, response.read()
        finally:
            conn.close()

    try:
        assert process.stderr.readline().startswith("KromaRec listening")
        idle = rss(process.pid)
        if observe:
            observe("idle", process.pid)
        assert json.loads(request("", "/")[2])["PlayerSettings"] == "https://settings.rec.test"
        assert json.loads(request("api", "/api/versioncheck/v4?v=20230414.01")[2])["VersionStatus"] == 0
        assert json.loads(request("api", "/api/versioncheck/v4?v=20250718")[2])["VersionStatus"] == 1
        assert not {"DataCollection", "BugReporting", "GameLogs"} & json.loads(request("", "/")[2]).keys()
        assert request("datacollection", "/data/event", "POST", "{}")[0] == 404
        assert request("datacollection", "/data/events", "POST", "[]")[0] == 404
        assert json.loads(request("api", "/statsigUserProperties", "POST")[2]) == {"success": False}
        assert json.loads(request("moderation", "/voice/config")[2]) == {"AccountId": "", "ApiKey": ""}
        def grant(**fields):
            return request("auth", "/connect/token", "POST", urlencode(fields),
                           **{"Content-Type": "application/x-www-form-urlencoded"})

        password = "temporary-testing-password"
        status, headers, body = grant(grant_type="create_account", password=password)
        assert status == 200 and headers["Cache-Control"] == "no-store"
        issued = json.loads(body)
        status, _, body = request("accounts", "/account/me", Authorization="Bearer " + issued["access_token"])
        assert status == 200
        account = json.loads(body)
        assert account["accountId"] == 2 and account["email"] == ""
        status, _, body = request("accounts", "/account/2")
        assert status == 200 and "email" not in json.loads(body)
        assert grant(grant_type="password", username=account["username"], password="incorrect")[0] == 400
        assert grant(grant_type="password", username=account["username"], password=password)[0] == 200
        with concurrent.futures.ThreadPoolExecutor(2) as pool:
            refresh_results = list(pool.map(
                lambda _: grant(grant_type="refresh_token", refresh_token=issued["refresh_token"])[0],
                range(2),
            ))
        assert sorted(refresh_results) == [200, 400]
        assert grant(grant_type="refresh_token", refresh_token=issued["refresh_token"])[0] == 400

        def settings(method="GET", body=None, token=None, mime="application/json"):
            return request("settings", "/playersettings", method, body, **{
                "Authorization": "Bearer " + (token or signed_token("42", int(time.time()) + 60)),
                "Content-Type": mime,
            })

        assert request("settings", "/playersettings")[0] == 401
        assert settings(token=signed_token("42", 1))[0] == 401
        assert settings(token=signed_token("042", int(time.time()) + 60))[0] == 401
        assert settings(token=signed_token("42", int(time.time()) + 60, "none"))[0] == 401
        form_header = {"Content-Type": "application/x-www-form-urlencoded"}
        a = "Bearer " + issued["access_token"]
        second = json.loads(grant(grant_type="create_account", password=password)[2])
        third = json.loads(grant(grant_type="create_account", password=password)[2])
        b = "Bearer " + second["access_token"]
        c = "Bearer " + third["access_token"]
        for verb, route, form in (
            ("PUT", "bio", {"bio": "A persistent bio"}),
            ("POST", "identityflags", {"identityFlags": "2"}),
            ("PUT", "personalpronouns", {"pronounFlags": "3"}),
        ):
            path = f"/account/me/{route}"
            assert request("accounts", path, verb, urlencode(form), **form_header)[0] == 401
            assert request("accounts", path, verb, urlencode(form), Authorization=a, **form_header)[0] == 200
        assert request("accounts", "/account/me/bio", "PUT", urlencode({"bio":"x"*256}), Authorization=a, **form_header)[0] == 400
        assert json.loads(request("accounts", "/account/2/bio")[2]) == {"accountId":2,"bio":"A persistent bio"}
        public = json.loads(request("accounts", "/account/2")[2])
        assert (public["identityFlags"], public["personalPronouns"]) == (2, 3)

        def social(path, token=a, method="GET", data=None, mime="application/json"):
            return request("api", "/api/" + path, method, data, Authorization=token,
                           **{"Content-Type":mime})

        def social_json(path, token=a, method="GET", data=None, mime="application/json"):
            status, _, body = social(path, token, method, data, mime)
            assert status == 200, (path, status, body)
            return json.loads(body)

        r = "relationships/"
        assert social(r+"v2/get", "")[0] == 401
        assert social(r+"v2/sendfriendrequest?id=2")[0] == 400
        assert social_json(r+"v2/sendfriendrequest?id=3")["RelationshipType"] == 1
        assert social_json(r+"v2/sendfriendrequest", a, "POST", '{"PlayerId":3}')["RelationshipType"] == 1
        assert social_json(r+"v2/get", b)[0]["RelationshipType"] == 2
        inbox = social_json("messages/v2/get", b)
        assert len(inbox) == 1 and inbox[0]["Type"] == 4 and inbox[0]["FromPlayerId"] == 2
        message_id = inbox[0]["Id"]
        assert social("messages/v3/delete", a, "POST", json.dumps([message_id]))[0] == 200
        assert len(social_json("messages/v2/get", b)) == 1
        assert social("messages/v3/delete", b, "POST", json.dumps({"MessageIds":[message_id]}))[0] == 200
        assert social_json("messages/v2/get", b) == []
        assert social_json(r+"v2/acceptfriendrequest", b, "POST", "PlayerId=2",
                           "application/x-www-form-urlencoded")["RelationshipType"] == 3
        assert social_json(r+"v2/acceptfriendrequest?id=2", b)["RelationshipType"] == 3
        assert social_json("messages/v2/get", a)[0]["Type"] == 40
        assert social_json(r+"v2/addfriend?id=4", a)["RelationshipType"] == 3
        assert social_json(r+"v2/addfriend?id=4", b, "POST")["RelationshipType"] == 3
        assert social_json(r+"mutualfriends?id=3")[0]["AccountId"] == 4
        for flag in ("ignore", "mute", "favorite"):
            assert social_json(r+f"v1/{flag}?id=3")["Success"]
            assert social_json(r+f"v1/{flag}", a, "POST",'{"PlayerId":3}')["Success"]
        assert social_json(r+"v2/removefriend?id=3", a, "POST")["RelationshipType"] == 0
        assert social_json(r+"v2/removefriend?id=2", b)["Ignored"] == 0
        assert all(social_json(r+"v2/get")[0][key] == 1 for key in ("Ignored","Muted","Favorited"))
        for flag in ("unignore", "unmute", "unfavorite"):
            assert social_json(r+f"v1/{flag}?id=3")["Success"]
            assert social_json(r+f"v1/{flag}", a, "POST", '{"PlayerId":3}')["Success"]
        assert social_json(r+"v2/sendfriendrequest?id=3")["RelationshipType"] == 1
        assert social_json(r+"v2/sendfriendrequest?id=2", b)["RelationshipType"] == 3

        def presence(route, token=b, lock="current"):
            return request("match", f"/player/{route}", "POST",
                           urlencode({"LoginLock":lock}), Authorization=token, **form_header)

        def online_count():
            return social_json("messages/v1/friendOnlineStatus", a, "POST")["value"]["FriendsOnlineCount"]

        assert presence("heartbeat", "")[0] == 401
        assert online_count() == 0
        assert presence("login")[0] == 200
        assert json.loads(presence("heartbeat")[2])["isOnline"]
        assert online_count() == 1
        assert presence("heartbeat", lock="superseded")[2] == b""
        if schema >= 5:
            assert json.loads(request("match","/matchmake/dorm","POST",Authorization=b)[2])["roomInstance"]["roomInstanceType"] == 2
        assert presence("logout")[0] == 200
        assert not json.loads(presence("heartbeat")[2])["isOnline"]
        assert online_count() == 0

        password_route = "/account/me/changepassword"
        replacement = password + "-new"
        assert request("auth",password_route,"POST",urlencode({"oldPassword":"bad","newPassword":replacement}),Authorization=b,**form_header)[0] == 400
        assert request("auth",password_route,"POST",urlencode({"oldPassword":password,"newPassword":replacement}),Authorization=b,**form_header)[0] == 200
        assert grant(grant_type="refresh_token", refresh_token=second["refresh_token"])[0] == 400
        assert grant(grant_type="password", account_id=3,password=password)[0] == 400
        status, _, body = grant(grant_type="password", account_id=3, password=replacement)
        assert status == 200
        if schema >= 7:
            assert request("accounts", "/account/me", Authorization=b)[0] == 401
            legacy = signed_token("3", int(time.time()) + 60)
            assert request("accounts", "/account/me", Authorization="Bearer " + legacy)[0] == 401
            renewed = "Bearer " + json.loads(body)["access_token"]
            assert request("accounts", "/account/me", Authorization=renewed)[0] == 200

        defaults = json.loads(settings()[2])
        assert [item["Value"] for item in defaults] == ["77", "11", "0"]
        assert settings("PUT", "key=Volume&value=0.5", mime="application/x-www-form-urlencoded")[0] == 200
        assert settings("PUT", '{"Key":"Growth.YIR.Ignore","Value":5}')[0] == 200

        def write(index):
            return settings("PUT", json.dumps({"Key": f"setting{index}", "Value": index}))[0]

        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            assert list(pool.map(write, range(20))) == [200] * 20
        stored = {item["Key"]: item["Value"] for item in json.loads(settings()[2])}
        assert stored["Volume"] == "0.5" and stored["setting19"] == "19" and len(stored) == 24
        assert settings("DELETE", '"Volume"')[0] == 200
        assert all(item["Key"] != "Volume" for item in json.loads(settings()[2]))
        import sqlite3
        with sqlite3.connect(work / "settings.sqlite3") as db:
            assert db.execute("PRAGMA journal_mode").fetchone() == ("wal",)
            assert db.execute("PRAGMA user_version").fetchone() == (schema,)

        status, headers, body = request("cdn", "/room/blob", Range="bytes=255-258")
        assert (status, headers["Content-Range"], body) == (206, f"bytes 255-258/{len(content)}", content[255:259])
        assert request("cdn", "/room/blob", Range="bytes=-4")[2] == content[-4:]
        assert request("cdn", "/room/blob", **{"If-None-Match": headers["ETag"]})[0] == 304
        assert request("cdn", "/room/blob", Range="bytes=99999999-")[0] == 416
        assert request("cdn", "/room/blob", "HEAD")[2] == b""
        assert hashlib.sha256(request("cdn", "/room/blob")[2]).digest() == hashlib.sha256(content).digest()
        assert request("cdn", "/room/%2e%2e/private")[0] == 400
        assert request("cdn", "/room/outside")[0] == 404
        assert request("cdn", "/config/RRPlusConfig_v3")[2] == (assets / "config" / "RRPlusConfig_v3.json").read_bytes()

        connections = []
        try:
            for _ in range(100):
                conn = http.client.HTTPConnection("127.0.0.1", port, timeout=4)
                connections.append(conn)
                conn.request("GET", "/", headers={"Host": "rec.test"})
                assert conn.getresponse().read()
            active = rss(process.pid)
            if observe:
                observe("warm", process.pid)
        finally:
            for conn in connections:
                conn.close()
        process.send_signal(signal.SIGTERM)
        assert process.wait(timeout=12) == 0
        return idle, active
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    parser.add_argument("--schema", type=int, default=8)
    args = parser.parse_args()
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory() as directory:
        idle, active = smoke(binary, Path(directory), args.schema)
    print(f"PASS | binary {binary.stat().st_size} bytes | idle RSS {idle} KiB | warm RSS with 100 keep-alives {active} KiB")
