"""Exercise first-time scene admission and a ten-actor loopback RecCenter room."""
import time
from contextlib import ExitStack

from luxon_smoke import APP, enter
from memory_rss import cpu_seconds
from smoke import rss


def run_ten(account, request, port, backend_pid, luxon_pid):
    users = [account() for _ in range(10)]

    def join(player, token, path):
        room = request("match", path, "POST", token)["roomInstance"]
        info = request("match", "/player/connection-info", token=token)["value"]
        assert room["photonRoomId"] == info["photonRoomId"]
        peer, actor = enter(port, str(player), room["photonRoomId"], APP, info["photonAuthToken"])
        return room, peer, actor

    for path, room_type in (("/matchmake/none", 0), ("/matchmake/dorm", 2)):
        room, peer, _ = join(*users[0], path)
        try:
            assert room["roomInstanceType"] == room_type
            if room_type == 0:
                assert room["roomId"] == 13
            metadata = request("rooms", f"/rooms/{room['roomId']}", token=users[0][1])
            assert any(s["UnitySceneId"] == room["location"] and s["SubRoomId"] == room["subRoomId"]
                       for s in metadata["SubRooms"])
            peer.operation(254, {})
        finally:
            peer.close()

    with ExitStack() as sockets:
        peers, actors, instances = [], set(), set()
        for player, token in users:
            room, peer, actor = join(player, token, "/matchmake/room/2")
            sockets.callback(peer.close)
            instances.add((room["roomInstanceId"], room["photonRoomId"]))
            actors.add(actor)
            peers.append((peer, actor))
        assert len(instances) == 1 and len(actors) == 10
        pids = (backend_pid, luxon_pid)
        start = time.monotonic()
        before = [cpu_seconds(pid) for pid in pids]
        for cycle in range(10):
            for index, (sender, actor) in enumerate(peers):
                payload = bytes([cycle, index])
                sender.send_operation(253, {244: 45, 245: payload})
                for receiver, _ in peers:
                    if receiver is not sender:
                        event = receiver.event(45)
                        assert event[254] == actor and event[245] == payload
            time.sleep(1)
        elapsed = time.monotonic() - start
        cpu = [cpu_seconds(pid) - base for pid, base in zip(pids, before)]
        print(f"PASS | Orientation and dorm wire admission; ten RecCenter actors, 900 deliveries in {elapsed:.3f}s"
              f" | RSS KiB {rss(backend_pid)}/{rss(luxon_pid)} | CPU seconds {cpu[0]:.3f}/{cpu[1]:.3f}")
        for peer, _ in peers:
            peer.operation(254, {})
