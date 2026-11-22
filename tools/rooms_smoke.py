#!/usr/bin/env python3
"""Verify rooms, session allocation, presence, and SignalR over loopback."""
import argparse
import base64
from concurrent.futures import ThreadPoolExecutor
import hashlib
import hmac
import http.client
import json
import os
from pathlib import Path
import socket
import sqlite3
import struct
import subprocess
import tempfile
import time
from urllib.parse import urlencode

from smoke import ROOT, SECRET, rss


class HubSocket:
    def __init__(self, port, token, connection_id, batched=False, tls=None):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=5)
        if tls:
            self.sock = tls.wrap_socket(self.sock, server_hostname="notify.rec.test")
        self.file = self.sock.makefile("rb")
        key = base64.b64encode(os.urandom(16)).decode()
        request = (
            f"GET /hub/v1?id={connection_id} HTTP/1.1\r\nHost: notify.rec.test\r\n"
            f"Authorization: Bearer {token}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n"
            f"Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\n\r\n"
        )
        self.sock.sendall(request.encode())
        assert b"101 Switching Protocols" in self.file.readline()
        headers = {}
        while (line := self.file.readline()) != b"\r\n":
            name, value = line.decode().split(":", 1)
            headers[name.lower()] = value.strip()
        expected = base64.b64encode(hashlib.sha1(
            (key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
        assert headers["sec-websocket-accept"] == expected
        handshake = {"protocol": "json", "version": 1}
        if batched:
            self.send(handshake, {"type": 6})
        else:
            self.send(handshake)
        assert self.receive() == {}
        assert self.receive()["target"] == "OnConnect"
        if batched:
            assert self.receive() == {"type": 6}

    def send(self, *values):
        payload = b"".join(json.dumps(value).encode() + b"\x1e" for value in values)
        assert len(payload) < 126
        mask = os.urandom(4)
        self.sock.sendall(bytes((0x81, 0x80 | len(payload))) + mask +
                          bytes(value ^ mask[i % 4] for i, value in enumerate(payload)))

    def receive(self):
        first, length = self.file.read(2)
        assert first == 0x81
        if length == 126:
            length = struct.unpack("!H", self.file.read(2))[0]
        return json.loads(self.file.read(length).rstrip(b"\x1e"))

    def close(self):
        self.file.close()
        self.sock.close()


def run(binary, baseline, baseline_schema=7, luxon_port=None, secure_luxon=None):
    with tempfile.TemporaryDirectory() as directory:
        db_path = Path(directory) / "test.sqlite3"
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        from luxon_smoke import APP
        env = {**os.environ, "KROMAREC_BIND": f"127.0.0.1:{port}", "KROMAREC_DOMAIN": "rec.test",
               "KROMAREC_ASSETS": str(ROOT / "assets"), "KROMAREC_DB": str(db_path),
               "KROMAREC_JWT_SECRET": SECRET, "KROMAREC_PASSWORD_SIGNUP": "true",
               "KROMAREC_PHOTON_REALTIME_APP_ID": APP,
               "KROMAREC_PHOTON_VOICE_APP_ID": "integration-test-voice", "KROMAREC_PHOTON_REGION": "us"}

        if secure_luxon:
            import secrets
            with socket.socket() as gate:
                gate.bind(("127.0.0.1",0))
                env["KROMAREC_PHOTON_GATE_ADDR"] = f"127.0.0.1:{gate.getsockname()[1]}"
            env["KROMAREC_PHOTON_GATE_KEY"] = secrets.token_hex(32)

        def request(service, path, method="GET", token=None, fields=None, expected=200):
            headers = {"Host": f"{service}.rec.test"}
            if token:
                headers["Authorization"] = f"Bearer {token}"
            body = None
            if fields is not None:
                headers["Content-Type"] = "application/x-www-form-urlencoded"
                body = urlencode(fields)
            conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
            try:
                conn.request(method, path, body, headers)
                response = conn.getresponse()
                data = response.read()
                if path == "/player/connection-info" and response.status == 200:
                    assert response.getheader("Cache-Control") == "no-store"
                assert response.status == expected, (method, path, response.status, data)
                return json.loads(data) if data and response.getheader("Content-Type", "").startswith("application/json") else data
            finally:
                conn.close()

        def start(executable):
            with (Path(directory) / "startup.log").open("a") as trace:
                process = subprocess.Popen([executable], env=env, stderr=trace)
            for _ in range(200):
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=.2):
                        return process
                except OSError:
                    if process.poll() is not None:
                        break
                    time.sleep(.02)
            process.kill()
            process.wait()
            raise RuntimeError("server did not start")

        def stop(process):
            process.terminate()
            try:
                assert process.wait(timeout=12) == 0
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()

        def grant(**fields):
            return request("auth", "/connect/token", "POST", fields=fields)["access_token"]

        password = "room-test-password"
        if baseline:
            process = start(baseline)
            try:
                old_credentials = request("auth", "/connect/token", "POST",
                                          fields={"grant_type": "create_account", "password": password})
                old_token = old_credentials["access_token"]
                old = request("accounts", "/account/me", token=old_token)
                request("playersettings", "/playersettings", "PUT", old_token, {"key": "Migrated", "value": "yes"})
                if baseline_schema >= 4:
                    prior_room = request("rooms", "/rooms/create", "POST",old_token,{"name":"PreMigrationRoom"})
                    prior_instance = request("match", f"/matchmake/room/{prior_room['RoomId']}",
                                             "POST", old_token)["roomInstance"]

            finally:
                stop(process)
            with sqlite3.connect(db_path) as db:
                assert db.execute("PRAGMA user_version").fetchone() == (baseline_schema,)

        env["KROMAREC_TRACE_STARTUP"] = "true"
        process = start(binary)
        sidecar = None
        log = None
        try:
            if secure_luxon:
                import re
                # Use a unique set of loopback UDP ports rather than sharing a live deployment.
                udp = [socket.socket(socket.AF_INET,socket.SOCK_DGRAM) for _ in range(3)]
                try:
                    for sock in udp: sock.bind(("127.0.0.1",0))
                    ports = [sock.getsockname()[1] for sock in udp]
                finally:
                    for sock in udp: sock.close()
                config = (ROOT / "packaging/luxon.config.example.yml").read_text()
                config = re.sub(r"505[568]",lambda m:str(ports[{"5058":0,"5055":1,"5056":2}[m[0]]]),config)
                (Path(directory) / "config.yml").write_text(config)
                log = (Path(directory) / "luxon.log").open("w+")
                sidecar = subprocess.Popen([secure_luxon],cwd=directory,env=env,stdout=log,stderr=log)
                luxon_port = ports[0]
            idle = rss(process.pid)
            def account():
                temporary = grant(grant_type="create_account", password=password)
                profile = request("accounts", "/account/me", token=temporary)
                player = profile["accountId"]
                token = grant(grant_type="password", account_id=player, password=password)
                request("match", "/player/login", "POST", token, {"LoginLock": str(player)})
                return player, token

            if baseline:
                assert request("accounts", "/account/me", token=old_token)["accountId"] == old["accountId"]
                rotated = grant(grant_type="refresh_token", refresh_token=old_credentials["refresh_token"])
                assert request("accounts", "/account/me", token=rotated)["accountId"] == old["accountId"]
                restored = grant(grant_type="password", account_id=old["accountId"], password=password)
                assert request("accounts", "/account/me", token=restored)["username"] == old["username"]
                if baseline_schema >= 4:
                    assert request("rooms",f"/rooms/{prior_room['RoomId']}",token=restored)["Name"] == "PreMigrationRoom"
                    migrated = request("match", "/matchmake/none", "POST", restored)["roomInstance"]
                    assert migrated == prior_instance
                    with sqlite3.connect(db_path) as db:
                        assert db.execute("SELECT matchmaking_policy FROM room_instance WHERE id=?",
                                          (migrated["roomInstanceId"],)).fetchone() == (0,)
                assert any(row["Key"] == "Migrated" and row["Value"] == "yes"
                           for row in request("playersettings", "/playersettings", token=restored))
            a_id, a = account()
            b_id, b = account()
            c_id, c = account()
            with sqlite3.connect(db_path) as db:
                assert db.execute("PRAGMA user_version").fetchone() == (8,)
                assert db.execute("PRAGMA journal_mode").fetchone() == ("wal",)
                assert db.execute("PRAGMA foreign_key_check").fetchall() == []
            request("api", "/api/config/v2?access_token=redact-query")
            request("api", "/sensitive-dynamic-path", expected=404)
            request("api", "/api/config/v2", "SECRET-METHOD", expected=404)
            request("api", "/api/PlayerReporting/v1/deviceId", "POST",
                    fields={"oldDeviceId":"redact-device"}, expected=404)
            startup = (Path(directory) / "startup.log").read_text()
            assert "startup api GET config 200" in startup
            assert "startup api POST device-id 404" in startup
            assert "startup api OTHER config 404" in startup
            assert not any(value in startup for value in (
                a,b,c,password,"redact-query","sensitive-dynamic-path","SECRET-METHOD","redact-device"
            ))
            picker = "/cachedlogin/forplatformid/0/76561198000000001"
            assert request("auth", picker) == []
            assert request("auth", picker, "POST", fields={"platformAuth":"unverified"}) == []
            request("auth", "/connect/token","POST", fields={"grant_type":"cached_login","account_id":a_id,
                    "platform_id":"76561198000000001"},expected=400)
            orientation = request("match", "/matchmake/none", "POST", b)["roomInstance"]
            assert orientation["roomId"] == 13 and orientation["roomInstanceId"] == -2
            assert orientation["location"] == "c79709d8-a31b-48aa-9eb8-cc31ba9505e8"
            request("match", "/player/logout", "POST", b)
            assert request("match", "/player/heartbeat","POST", b)["roomInstance"]["roomInstanceId"] == -2
            assert request("match", "/player/connection-info", token=b)["value"]["photonRoomId"] == "rec.13"
            assert request("rooms", "/photon_access_token", token=b)["RoomInstanceId"] == -2
            assert request("rooms", "/rooms/13")["Name"] == "Orientation"
            dorm_a = request("rooms", "/dormroom/me", token=a)
            dorm_b = request("match", "/matchmake/dorm", "POST", b)["roomInstance"]
            assert dorm_a != dorm_b["roomId"] and dorm_b["roomInstanceType"] == 2
            assert request("match", "/matchmake/dorm", "POST", a)["roomInstance"]["roomId"] == dorm_a
            assert request("rooms", f"/rooms/{dorm_a}", token=a)["IsDorm"]
            assert request("rooms", f"/rooms/{dorm_a}", token=b, expected=404)
            base = request("rooms", "/rooms/base")
            template = next(room for room in base if room["Name"] == "MakerRoom")
            made = request("rooms", f"/rooms/{template['RoomId']}/clone", "POST", a, {"name": "TwoPlayers"})
            room = made["value"]
            assert made["success"] and room["Accessibility"] == 0
            assert room["SubRooms"][0]["UnitySceneId"] == template["SubRooms"][0]["UnitySceneId"]
            studio = request("rooms", "/rooms/create", "POST", a, {"name": "PersistentRoom"})
            rid = room["RoomId"]
            path = f"/rooms/{rid}"
            assert request("rooms", "/rooms/create", "POST", b, {"name": "twoplayers"}, expected=409)
            assert request("rooms", "/rooms?name=TwoPlayers") == {}
            assert [entry["RoomId"] for entry in request("rooms",f"/rooms/bulk?id=2&id={rid},13&id=2")] == [2,13]
            assert [entry["RoomId"] for entry in request("rooms","/rooms/bulk","POST",a,[("id",rid),("id",2)])] == [rid,2]
            assert request("rooms","/rooms/bulk","POST",a,[("id",rid),("ExcludePrivateRooms","True")]) == []
            request("rooms","/rooms/bulk",expected=400)
            request("rooms","/rooms/bulk?id=" + ",".join(["2"] * 101),expected=400)
            assert request("match", f"/matchmake/room/{rid}", "POST", b)["result"] == 25
            assert request("match", "/matchmake/room/DoesNotExist", "POST", b)["result"] == 20
            assert not request("rooms", path + "/description", "PUT", b, {"description": "bad"})["Success"]
            assert request("rooms", path + "/description", "PUT", a, {"description": "test room"})["Success"]
            assert request("rooms", path + "/name", "PUT", a, {"name": "TwoPlayersRenamed"})["Success"]
            assert request("rooms", path + f"/roles/{b_id}/invite", "PUT", a, {"role":30})["success"]
            assert any(message["Type"] == 62 and message["Data"] == "30" for message in request("api", "/api/messages/v2/get", token=b))
            assert request("rooms", path + f"/roles/{b_id}", "PUT", b, {"role":30})["success"]
            assert any(row["AccountId"] == b_id and row["Role"] == 30 for row in request("rooms", path + "/roles", token=a))
            assert request("rooms", path + "/accessibility", "PUT", b, {"accessibility":1})["success"]
            assert request("rooms", "/rooms/search?query=players%20%23community")["Results"][0]["RoomId"] == rid
            assert request("rooms", "/rooms/search/name?query=twoplayers")[0]["RoomId"] == rid
            assert request("rooms", f"/rooms?id={rid}")["Description"] == "test room"
            for listing, token in [("/roomserver/rooms/createdby/me", a), ("/rooms/createdby/me", a),
                                   ("/rooms/ownedby/me", b), ("/rooms/contributedby/me", b)]:
                assert rid in [x["RoomId"] for x in request("rooms", listing, token=token)]
            assert request("rooms", f"/rooms/{studio['RoomId']}/creator", "PUT", a, {"accountId":b_id})["value"]["CreatorAccountId"] == b_id
            for one, other in ((a,b_id),(b,a_id)):
                request("api", f"/api/relationships/v2/sendfriendrequest?id={other}", token=one)
            # Two independent clients race into the same newly published room.
            match_path = f"/matchmake/room/{rid}"
            correlation = "12345678-1234-4234-9234-123456789abc"
            with ThreadPoolExecutor(2) as pool:
                results = list(pool.map(lambda token: request("match", match_path, "POST", token, {"CorrelationId":correlation}), (a,b)))
            assert all(result["result"] == 0 and result["correlationId"] == correlation for result in results)
            first, second = [result["roomInstance"] for result in results]
            iid = first["roomInstanceId"]
            assert iid == second["roomInstanceId"] and first["photonRoomId"] == second["photonRoomId"]
            assert (first["location"], first["photonRegion"]) == (second["location"], second["photonRegion"])
            assert first["location"] and first["photonRoomId"]
            assert request("match", f"/room/{rid}/instances", token=a)[0]["playerIds"] == sorted([a_id,b_id])
            assert request("match", f"/room/{rid}/instances", token=c, expected=403) == b""
            assert request("match", f"/matchmake/instance/{iid}", "POST", c)["result"] == 20
            assert request("match", f"/matchmake/instance/{iid}", "POST", a)["roomInstance"]["roomInstanceId"] == iid
            assert request("match", f"{match_path}/{room['SubRooms'][0]['SubRoomId']}", "POST", a)["roomInstance"]["roomInstanceId"] == iid
            assert request("match","/player/exclusivelogin","POST",a,{"LoginLock":"exclusive"}) == {"errorCode":0}
            request("match","/player/verifylock","POST",a,{"LoginLock":a_id},expected=409)
            request("match","/player/verifylock","POST",a,{"LoginLock":"exclusive"})
            before_stale_logout = request("match", "/player/connection-info", token=a)["value"]["photonAuthToken"]
            request("match","/player/logout","POST",a,{"LoginLock":a_id})
            request("match","/player/verifylock","POST",a,{"LoginLock":"exclusive"})
            assert request("match","/player/connection-info",token=a)["value"]["photonRoomId"] == first["photonRoomId"]
            request("match","/player/login","POST",a,{"LoginLock":a_id})
            tickets = []
            for player, token in ((a_id,a),(b_id,b)):
                info = request("match", "/player/connection-info", token=token)["value"]
                assert info["photonRoomId"] == first["photonRoomId"] and info["photonRegion"] == "us"
                assert info["photonRealtimeAppId"] == APP
                tickets.append(info["photonAuthToken"])
                signed, signature = info["photonAuthToken"].rsplit(".", 1)
                decoded = json.loads(base64.urlsafe_b64decode(signed.split(".")[1] + "==="))
                assert decoded["sub"] == str(player) and decoded["aud"] == info["photonRealtimeAppId"]
                assert hmac.compare_digest(hmac.digest(SECRET.encode(),signed.encode(),"sha256"),
                                           base64.urlsafe_b64decode(signature + "==="))
                request("accounts", "/account/me", token=info["photonAuthToken"], expected=401)
                assert request("rooms", "/photon_access_token", token=token)["RoomInstanceId"] == iid
                assert request("match", "/player/heartbeat", "POST", token, {"LoginLock":player})["roomInstance"]["roomInstanceId"] == iid

            if luxon_port:
                from luxon_smoke import run as transport
                def footprint():
                    backend, luxon = rss(process.pid), rss(sidecar.pid)
                    print(f"two wire peers | KromaRec {backend} KiB | Luxon {luxon} KiB | combined {backend+luxon} KiB")
                transport(luxon_port, first["photonRoomId"], str(a_id), str(b_id), APP,
                          tickets if secure_luxon else (None,None), footprint if secure_luxon else None)
            if secure_luxon:
                from luxon_smoke import Peer, authenticate, enter
                from smoke import signed_token
                def reject(ticket, app=APP):
                    peer = Peer(("127.0.0.1",luxon_port))
                    try:
                        authenticate(peer,str(a_id),app,ticket,reject=True)
                    finally:
                        peer.close()
                def issue(token):
                    return request("match","/player/connection-info",token=token)["value"]["photonAuthToken"]
                def sign(fields):
                    encoded = base64.urlsafe_b64encode(json.dumps(fields).encode()).rstrip(b"=").decode()
                    signed = tickets[0].split(".")[0] + "." + encoded
                    mac = base64.urlsafe_b64encode(hmac.digest(SECRET.encode(),signed.encode(),"sha256")).rstrip(b"=")
                    return signed + "." + mac.decode()
                reject(None)
                reject(a)
                reject(tickets[0] + "bad")
                reject(tickets[0], "another-app")
                claims = json.loads(base64.urlsafe_b64decode(tickets[0].split(".")[1] + "==="))
                reject(sign({**claims,"exp":1}))
                reject(sign({**claims,"kromarec_use":"http"}))
                reject(sign({**claims,"aud":"another-app"}))
                reject(sign({k: v for k, v in claims.items() if not k.startswith("kromarec_") or k == "kromarec_use"}))
                reject(signed_token(str(a_id),int(time.time())+60))
                spoof = Peer(("127.0.0.1",luxon_port))
                try:
                    assert authenticate(spoof,"999",APP,before_stale_logout,operation=231)[225] == str(a_id)
                    spoof.disconnect()
                finally:
                    spoof.close()
                # Keep B connected while revoking A. Assert a server ENet disconnect
                # and the matching actor-leave on B, then exchange a fresh event.
                observer, _ = enter(luxon_port, str(b_id), first["photonRoomId"], APP, issue(b))
                def active_eviction(token, player, action, timeout):
                    victim, actor = enter(luxon_port, str(player), first["photonRoomId"], APP, issue(token))
                    try:
                        victim.send_operation(253, {244:44, 245:b"ready"})
                        assert observer.event(44)[254] == actor
                        action()
                        victim.expect_disconnect(timeout)
                        assert observer.event(254)[254] == actor
                    finally:
                        victim.close()

                try:
                    active_eviction(a, a_id, lambda: request("match", "/player/logout", "POST", a), 4)
                    request("match", match_path, "POST", a)
                    def ban_a():
                        with sqlite3.connect(db_path) as db:
                            db.execute("INSERT INTO photon_ban(player) VALUES(?)", (a_id,))
                    active_eviction(a, a_id, ban_a, 9)
                    with sqlite3.connect(db_path) as db:
                        db.execute("DELETE FROM photon_ban WHERE player=?", (a_id,))
                    victim_id, victim_token = account()
                    request("match", match_path, "POST", victim_token)
                    hub_victim = HubSocket(port, victim_token, "live")
                    try:
                        def rotate_password():
                            request("auth", "/account/me/changepassword", "POST", victim_token,
                                    {"oldPassword":password,"newPassword":password+"-active"})
                        active_eviction(victim_token, victim_id, rotate_password, 4)
                        assert hub_victim.file.read(1) == b""
                    finally:
                        hub_victim.close()
                    request("accounts", "/account/me", token=victim_token, expected=401)
                    fresh_token = grant(grant_type="password", account_id=victim_id, password=password+"-active")
                    fresh, actor = enter(luxon_port, str(victim_id), first["photonRoomId"], APP, issue(fresh_token))
                    try:
                        # Cross the 12s watchdog with healthy leases and measure
                        # both processes while exchanging one round trip/second.
                        from memory_rss import cpu_seconds
                        started = time.monotonic()
                        cpu = [cpu_seconds(pid) for pid in (process.pid, sidecar.pid)]
                        for sequence in range(15):
                            payload = bytes([sequence])
                            fresh.send_operation(253, {244:45, 245:payload})
                            event = observer.event(45)
                            assert event[254] == actor and event[245] == payload
                            observer.send_operation(253, {244:46, 245:payload})
                            assert fresh.event(46)[245] == payload
                            time.sleep(max(0, started + sequence + 1 - time.monotonic()))
                        elapsed = time.monotonic() - started
                        print(f"15 bidirectional exchanges in {elapsed:.2f}s | CPU seconds: "
                              f"KromaRec {cpu_seconds(process.pid)-cpu[0]:.3f}, "
                              f"Luxon {cpu_seconds(sidecar.pid)-cpu[1]:.3f}")
                        fresh.disconnect()
                        observer.event(254)
                    finally:
                        fresh.close()
                    request("match", "/player/logout", "POST", fresh_token)
                finally:
                    observer.disconnect()
                    observer.close()
                print("PASS | live password/logout/ban eviction, observer continuity and fresh admission")
                private = request("match",match_path,"POST",a,{"JoinMode":2})["roomInstance"]
                enter(luxon_port,str(c_id),private["photonRoomId"],APP,issue(c),reject_join=True)
                from luxon_smoke import endpoint
                # A token already issued by Luxon cannot carry a banned user into Master.
                name_peer = Peer(("127.0.0.1",luxon_port))
                try:
                    handoff = authenticate(name_peer,str(c_id),APP,issue(c))
                    name_peer.handoff()
                    with sqlite3.connect(db_path) as db:
                        db.execute("INSERT INTO photon_ban(player) VALUES(?)",(c_id,))
                    master_peer = Peer(endpoint(handoff[230]))
                    try:
                        master_peer.operation(230,{221:handoff[221]},reject=True)
                    finally:
                        master_peer.close()
                finally:
                    name_peer.close()
                    with sqlite3.connect(db_path) as db:
                        db.execute("DELETE FROM photon_ban WHERE player=?",(c_id,))

                logout_ticket = issue(a)
                def revoke():
                    request("match","/player/logout","POST",a)
                enter(luxon_port,str(a_id),private["photonRoomId"],APP,logout_ticket,before_game_join=revoke)
                reject(logout_ticket)
                # Reserving the same room again does not make the old ticket reusable.
                reallocated = request("match", match_path, "POST", a)["roomInstance"]
                assert reallocated["photonRoomId"] == first["photonRoomId"]
                rejoined, _ = enter(luxon_port, str(a_id), reallocated["photonRoomId"], APP, issue(a))
                rejoined.disconnect()
                rejoined.close()
                reject(logout_ticket)

                # Password changes revoke the original ticket and an in-flight Game handoff.
                password_ticket = issue(c)
                c_instance = request("match", match_path, "POST", c)["roomInstance"]
                replacement = password + "-rotated"

                def change_password():
                    request("auth", "/account/me/changepassword", "POST", c,
                            {"oldPassword": password, "newPassword": replacement})
                enter(luxon_port, str(c_id), c_instance["photonRoomId"], APP, password_ticket,
                      before_game_auth=change_password)
                reject(password_ticket)
                request("match", "/player/connection-info", token=c, expected=401)
                c = grant(grant_type="password", account_id=c_id, password=replacement)
                password_rejoin, _ = enter(luxon_port, str(c_id), c_instance["photonRoomId"],
                                           APP, issue(c))
                password_rejoin.disconnect()
                password_rejoin.close()
                request("match", "/player/logout", "POST", c)
                with sqlite3.connect(db_path) as db:
                    db.execute("INSERT INTO photon_ban(player) VALUES(?)",(b_id,))
                reject(tickets[1])
                assert request("match",match_path,"POST",b)["result"] == 55
                # Revocation must also invalidate an expired live reservation.
                with sqlite3.connect(db_path) as db:
                    db.execute("UPDATE room_location SET expires=unixepoch()-1 WHERE player=?",(a_id,))
                enter(luxon_port,str(a_id),first["photonRoomId"],APP,issue(a),reject_join=True)
                with sqlite3.connect(db_path) as db:
                    db.execute("DELETE FROM photon_ban WHERE player=?",(b_id,))
                for token in (a,b):
                    request("match",match_path,"POST",token)
                log.flush()
                log.seek(0)
                output = log.read()
                assert all(ticket not in output for ticket in (*tickets, logout_ticket, password_ticket))
                assert env["KROMAREC_PHOTON_GATE_KEY"] not in output
                diagnostics = (Path(directory) / "startup.log").read_text()
                for stage in ("name-auth", "master-auth", "game-auth", "game-admission"):
                    assert f"photon stage={stage} admitted=true" in diagnostics
                assert "photon stage=game-admission admitted=false" in diagnostics
                assert "scene=c79709d8-a31b-48aa-9eb8-cc31ba9505e8" in diagnostics
                assert all(secret not in diagnostics for secret in (*tickets, c, password, env["KROMAREC_PHOTON_GATE_KEY"]))
                log.seek(0, 2)
                print("PASS | Luxon rejects invalid, expired, banned and unauthorized joins at the network boundary")

            negotiation = request("notify", "/hub/v1/negotiate?negotiateVersion=1","POST", b)
            from smoke import signed_token
            limited = HubSocket(port, signed_token(str(b_id), int(time.time()) + 2), "expiry")
            try:
                assert limited.file.read(1) == b""
            finally:
                limited.close()
            logout_hub = HubSocket(port, a, "logout")
            try:
                request("match", "/player/logout", "POST", a)
                assert logout_hub.file.read(1) == b""
            finally:
                logout_hub.close()
            request("match", match_path, "POST", a)
            old_hub = HubSocket(port,b,negotiation["connectionToken"])
            old_hub.close()
            negotiation = request("notify", "/hub/v1/negotiate?negotiateVersion=1","POST", b)
            hub = HubSocket(port,b,negotiation["connectionToken"],batched=True)
            try:
                hub.send({"type":6})
                assert hub.receive() == {"type":6}
                ws_rss = rss(process.pid)
                assert request("rooms", path + "/description", "PUT", a, {"description":"live edit"})["Success"]
                update = json.loads(hub.receive()["arguments"][0])
                assert update["Id"] == "RoomUpdate" and update["Msg"]["Description"] == "live edit"
                private = request("match", match_path, "POST", a, {"JoinMode":2})["roomInstance"]
                assert private["isPrivate"] and private["roomInstanceId"] != iid
                push = json.loads(hub.receive()["arguments"][0])
                assert push["Id"] == "PresenceUpdate"
                assert push["Msg"]["roomInstance"]["photonRoomId"] == ""
                assert request("match", "/player", "POST", b, [("id",a_id),("id",c_id)])[0]["roomInstance"]["photonRoomId"] == ""
                assert request("match", f"/player?id={a_id}", token=c)[0]["roomInstance"] is None
                private_id = private["roomInstanceId"]
                request("match", "/invite", "POST", c, {"playerId":b_id,"roomInstanceId":private_id}, expected=403)
                invite = request("match", "/invite", "POST", a, {"playerId":b_id,"roomInstanceId":private_id})
                push = json.loads(hub.receive()["arguments"][0])
                assert push["Id"] == "2" and push["Msg"]["Data"] == str(private_id)
                assert push["Msg"]["Id"] == invite["RoomInviteId"]
                assert request("match", f"/matchmake/player/{a_id}", "POST", b)["roomInstance"]["roomInstanceId"] == private_id
                request("match", f"/roominstance/{private_id}/reportjoinresult", "POST", b)
                request("match", f"/roominstance/{private_id}/inprogress", "PUT", b, {"inProgress":"true"})
                assert request("match", "/matchmake/none","POST",a)["roomInstance"]["isInProgress"]
                request("match", "/player/logout","POST",a)
                while True:
                    offline = json.loads(hub.receive()["arguments"][0])
                    if offline["Msg"]["roomInstance"] is None:
                        break
                assert offline["Msg"]["isOnline"] is False
                assert request("match", "/player/connection-info", token=a)["value"]["photonRoomId"] == ""
                assert request("match", f"/matchmake/instance/{private_id}", "POST", a)["roomInstance"]["photonRoomId"] == private["photonRoomId"]
            finally:
                hub.close()

            # The 20230414 request repeats AdditionalPlayerIds as separate form entries.
            request("match", "/matchmake/dorm", "POST", a)
            request("match", "/matchmake/dorm", "POST", b)
            request("match", "/matchmake/dorm", "POST", c)
            party_hubs = []
            try:
                for token in (b,c):
                    link = request("notify", "/hub/v1/negotiate?negotiateVersion=1", "POST", token)
                    party_hubs.append(HubSocket(port, token, link["connectionToken"]))
                fields = [("JoinMode", 2), ("AdditionalPlayerIds", a_id),
                          ("AdditionalPlayerIds", b_id), ("additionalplayerids", b_id),
                          ("AdditionalPlayerIds", c_id), ("AdditionalPlayerIds", "bogus")]
                destination = request("match", match_path, "POST", a, fields)["roomInstance"]
                dest_id = destination["roomInstanceId"]
                assert destination["isPrivate"]
                for sub, recipient in zip(party_hubs, (b_id,c_id)):
                    while True:
                        push = json.loads(sub.receive()["arguments"][0])
                        if push["Id"] == "2":
                            break
                    assert push["Msg"]["ToPlayerId"] == recipient
                    assert push["Msg"]["Type"] == 0 and push["Msg"]["Data"] == str(dest_id)
                request("match", match_path, "POST", a, fields)
                with sqlite3.connect(db_path) as db:
                    assert db.execute("SELECT recipient FROM room_invite WHERE sender=? AND instance=? ORDER BY recipient",
                                      (a_id, dest_id)).fetchall() == [(b_id,), (c_id,)]
                if secure_luxon:
                    enter(luxon_port, str(c_id), destination["photonRoomId"], APP, issue(c),
                          reject_join=True)
                for token in (b,c):
                    joined = request("match", f"/matchmake/player/{a_id}", "POST", token)["roomInstance"]
                    assert joined["roomInstanceId"] == dest_id
                if secure_luxon:
                    transport(luxon_port, destination["photonRoomId"], str(a_id), str(b_id),
                              APP, (issue(a),issue(b)))
            finally:
                for sub in party_hubs:
                    sub.close()

            # Four clients share the capacity; the fifth is allocated an overflow instance.
            # The seeded MakerRoom's capacity comes from RecFlare. Use the
            # four-player Studio room to test the known capacity boundary.
            assert request("rooms", f"/rooms/{studio['RoomId']}/accessibility", "PUT", b, {"accessibility":1})["success"]
            overflow_path = f"/matchmake/room/{studio['RoomId']}"
            _, d = account()
            _, e = account()
            a_public = request("match", overflow_path, "POST", a)["roomInstance"]
            for token in (b,c,d):
                full = request("match", overflow_path, "POST", token)["roomInstance"]
                assert full["roomInstanceId"] == a_public["roomInstanceId"]
            assert full["isFull"]
            overflow = request("match", overflow_path, "POST", e)["roomInstance"]["roomInstanceId"]
            full_id = full["roomInstanceId"]
            assert overflow != full_id
            policy = f"/roominstance/{full_id}/matchpolicy"
            request("match", policy, "PUT", e, {"policy":1}, expected=403)
            request("match", policy, "PUT", b, {"policy":3}, expected=400)
            request("match", policy, "PUT", b, {"policy":1})
            request("match", "/matchmake/dorm", "POST", b)
            assert request("match", overflow_path, "POST", b)["roomInstance"]["roomInstanceId"] == overflow
            request("match", f"/roominstance/{overflow}/matchpolicy", "PUT", e, {"policy":2})
            assert request("match", overflow_path, "POST", e)["roomInstance"]["roomInstanceId"] == overflow
            request("match", "/matchmake/dorm", "POST", b)
            assert request("match", overflow_path, "POST", b)["roomInstance"]["roomInstanceId"] == full_id
            close = f"/roominstance/{full_id}/markprivate"
            request("match", close, "POST", c, expected=403)
            request("match", close, "POST", b)
            closed = request("match", "/matchmake/none", "POST", c)["roomInstance"]
            assert closed["isPrivate"] and closed["roomInstanceId"] == full_id
            request("match", "/matchmake/dorm", "POST", c)
            assert request("match", overflow_path, "POST", c)["roomInstance"]["roomInstanceId"] not in (full_id,overflow)
            # Force expiry and confirm it is swept before allocating another instance.
            with sqlite3.connect(db_path) as db:
                db.execute("UPDATE room_instance SET touched=unixepoch()-1000")
                db.execute("UPDATE room_location SET expires=unixepoch()-1000")
            recalculated = request("match", match_path, "POST", a)["roomInstance"]
            assert recalculated["roomInstanceId"] != iid
            assert not request("rooms", path, "DELETE", c)["Success"]
            deleted_actor = None
            if secure_luxon:
                deleted_actor, _ = enter(luxon_port, str(a_id), recalculated["photonRoomId"], APP, issue(a))
            assert request("rooms", path, "DELETE", a)["Success"]
            if deleted_actor:
                try:
                    deleted_actor.expect_disconnect(9)
                finally:
                    deleted_actor.close()
            request("rooms", path, expected=404)
            assert request("match", f"/room/{rid}/instances", token=a, expected=404)
            with sqlite3.connect(db_path) as db:
                assert db.execute("SELECT count(*) FROM room_instance WHERE room_id=?", (rid,)).fetchone() == (0,)
                assert db.execute("PRAGMA integrity_check").fetchone() == ("ok",)
            if secure_luxon:
                dorm = request("match", "/matchmake/dorm", "POST", a)["roomInstance"]
                stalled, _ = enter(luxon_port, str(a_id), dorm["photonRoomId"], APP, issue(a))
                import signal
                try:
                    process.send_signal(signal.SIGSTOP)
                    stalled.expect_disconnect(16)
                finally:
                    process.send_signal(signal.SIGCONT)
                    stalled.close()
                shutdown_peer, _ = enter(luxon_port, str(a_id), dorm["photonRoomId"], APP, issue(a))
                try:
                    stop(process)
                    shutdown_peer.expect_disconnect(4)
                finally:
                    shutdown_peer.close()
                print("PASS | active room deletion, stalled backend and clean shutdown disconnect Game peer")
        finally:
            if sidecar is not None:
                sidecar.terminate()
                sidecar.wait(timeout=5)
            if log is not None: log.close()
            stop(process)
        process = start(binary)
        try:
            persisted = request("rooms", f"/rooms/{studio['RoomId']}", token=b)
            assert persisted["CreatorAccountId"] == b_id and persisted["Name"] == "PersistentRoom"
            assert request("rooms", "/dormroom/me", token=a) == dorm_a
            assert request("match", "/matchmake/none", "POST", a)["roomInstance"]["roomId"] == dorm_a
        finally:
            stop(process)
        print(f"PASS | rooms, migration, session allocation, privacy, SignalR, restart | idle {idle} KiB | after joins with SignalR open {ws_rss} KiB")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--baseline-schema", type=int, default=7)
    parser.add_argument("--luxon-port", type=int)
    parser.add_argument("--secure-luxon", type=Path)
    args = parser.parse_args()
    run(args.binary.resolve(), args.baseline.resolve() if args.baseline else None, args.baseline_schema, args.luxon_port, args.secure_luxon.resolve() if args.secure_luxon else None)
