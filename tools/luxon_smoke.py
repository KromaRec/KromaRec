#!/usr/bin/env python3
"""Exercise an actual Luxon Name → Master → Game chain over loopback UDP."""
import argparse
from collections import deque
import secrets
import socket
import struct
from time import monotonic

APP = "7a6eafd9-8a49-47bd-98da-c743525a3e72"


class Reader:
    def __init__(self, data):
        self.data = data
        self.offset = 0

    def byte(self):
        result = self.data[self.offset]
        self.offset += 1
        return result

    def varint(self):
        result = 0
        for shift in range(0, 35, 7):
            byte = self.byte()
            result |= (byte & 127) << shift
            if byte < 128:
                return result
        raise ValueError("invalid varint")

    def value(self):
        tag = self.byte()
        if tag in (7, 67):
            size = self.varint()
            result = self.data[self.offset:self.offset + size]
            self.offset += size
            return result.decode() if tag == 7 else result
        if tag in (3, 11):
            return self.byte()
        if tag == 9:
            value = self.varint()
            return (value >> 1) ^ -(value & 1)
        if tag in (30, 34):
            return 0
        if tag == 8:
            return None
        if tag in (21, 73, 71):
            count = self.varint()
            if tag == 21:
                return {self.value(): self.value() for _ in range(count)}
            if tag == 73:
                return [self.varint() for _ in range(count)]
            return [self.string() for _ in range(count)]
        if tag in (27, 28):
            return tag == 28
        raise ValueError(f"unsupported probe value tag {tag}")

    def string(self):
        size = self.varint()
        result = self.data[self.offset:self.offset + size].decode()
        self.offset += size
        return result

    def fields(self):
        return {self.byte(): self.value() for _ in range(self.byte())}


def encode(value):
    if isinstance(value, int):
        return bytes((3, value))
    tag = 7 if isinstance(value, str) else 67
    payload = value.encode() if isinstance(value, str) else value
    length = len(payload)
    prefix = bytearray()
    while length >= 128:
        prefix.append((length & 127) | 128)
        length >>= 7
    prefix.append(length)
    return bytes((tag,)) + prefix + payload


class Peer:
    def __init__(self, address):
        self.sock = socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
        self.sock.connect(address)
        self.challenge = secrets.randbits(32)
        self.peer_id = -1
        self.disconnect_acked = False
        self.disconnected = False
        self.seq = 0
        self.messages = deque()
        self.received = set()
        payload = bytearray(32)
        payload[2:4] = struct.pack("!H",1200)
        for idx,val in [(6,128),(11,2),(22,19),(23,136),(27,2),(31,2)]:
            payload[idx] = val
        self.send(2,255,1,1,payload)
        self.wait(lambda msg: msg is None)
        self.reliable(bytes((243,0,1,8,0,0,0,0,0)) + b"kromarec".ljust(32,b"\0"))
        self.wait(lambda msg: msg is not None and msg[:2] == bytes((243,1)))

    def send(self, code, channel, flags, seq, payload):
        self.sock.send(struct.pack("!hBBII",self.peer_id,0,1,int(monotonic()*1000) & 0xffffffff,self.challenge)
                       + struct.pack("!BBBBII",code,channel,flags,4,12+len(payload),seq) + payload)

    def reliable(self, data):
        self.seq += 1
        self.send(6,0,1,self.seq,data)

    def read(self, deadline):
        self.sock.settimeout(max(.001,deadline-monotonic()))
        packet = self.sock.recv(4096)
        _, kind, count, timestamp, challenge = struct.unpack_from("!hBBII",packet)
        assert kind == 0 and challenge == self.challenge
        offset = 12
        for _ in range(count):
            code,channel,flags,_,length,seq = struct.unpack_from("!BBBBII",packet,offset)
            body = packet[offset+12:offset+length]
            offset += length
            if code == 1 and channel == 255 and struct.unpack_from("!I", body)[0] == 2:
                self.disconnect_acked = True
            if code == 4:
                self.disconnected = True
            if code == 3:
                self.peer_id = struct.unpack_from("!h",body)[0]
            if flags == 1:
                self.send(1,channel,0,seq,struct.pack("!II",seq,timestamp))
            if code in (3,6) and (channel,seq) not in self.received:
                self.received.add((channel,seq))
                self.messages.append(None if code == 3 else body)

    def expect_disconnect(self, timeout):
        deadline = monotonic() + timeout
        while not self.disconnected:
            if monotonic() >= deadline:
                raise TimeoutError("expected server-initiated ENet disconnect")
            self.read(deadline)

    def wait(self, predicate):
        deadline = monotonic()+4
        while monotonic() < deadline:
            if not self.messages:
                self.read(deadline)
            while self.messages:
                msg = self.messages.popleft()
                if predicate(msg):
                    return msg
        raise TimeoutError("expected Luxon response/event")

    def send_operation(self, code, fields):
        self.reliable(bytes((243,2,code,len(fields))) +
                      b"".join(bytes((k,))+encode(v) for k,v in fields.items()))

    def operation(self, code, fields, reject=False):
        self.send_operation(code, fields)
        data = self.wait(lambda msg: msg and msg[:3] == bytes((243,3,code)))
        status = struct.unpack_from("<h",data,3)[0]
        if reject:
            assert status != 0, (code, status)
            return status
        assert status == 0, (code,status)
        result = Reader(data[5:])
        result.value()
        return result.fields()

    def event(self, code):
        data = self.wait(lambda msg: msg and msg[:3] == bytes((243,4,code)))
        return Reader(data[3:]).fields()

    def handoff(self):
        self.disconnect()
        deadline = monotonic() + 4
        while not self.disconnect_acked:
            if monotonic() >= deadline:
                raise TimeoutError("expected acknowledgment before handoff")
            self.read(deadline)

    def disconnect(self):
        self.send(4,255,1,2,b"")

    def close(self):
        self.sock.close()


def endpoint(address):
    host,port = address.rsplit(":",1)
    if host != "127.0.0.1":
        raise ValueError("probe accepts loopback routing only")
    return host,int(port)


def get_regions(name, app):
    fields = name.operation(220, {224: app})
    assert len(fields[210]) == len(fields[230])
    return dict(zip(fields[210], fields[230]))


def authenticate(name, user, app, ticket, reject=False, operation=230):
    fields = {224:app,220:"20230414",225:user}
    if ticket:
        fields[216] = "token=" + ticket
        fields[217] = 0  # Photon CustomAuthenticationType.Custom
    return name.operation(operation,fields,reject)


def enter(name_port, user, room, app, ticket=None, reject_join=False,
          before_game_join=None, before_game_auth=None):
    with_peers = []
    try:
        name = Peer(("127.0.0.1",name_port))
        with_peers.append(name)
        regions = get_regions(name, app) if ticket else {}
        reply = authenticate(name,user,app,ticket)
        if ticket:
            # The signed subject replaces the untrusted UserId field.
            assert reply[225] == user
            assert regions["us"] == reply[230]
        name.handoff()
        master = Peer(endpoint(reply[230]))
        with_peers.append(master)
        master.operation(230,{221:reply[221]})
        if reject_join:
            master.operation(226,{255:room,215:1},reject=True)
            master.disconnect()
            return
        reply = master.operation(226,{255:room,215:1})
        master.handoff()
        game = Peer(endpoint(reply[230]))
        with_peers.append(game)
        if before_game_auth:
            before_game_auth()
            game.operation(230, {221: reply[221]}, reject=True)
            game.disconnect()
            return
        game.operation(230,{221:reply[221]})
        if before_game_join:
            before_game_join()
            game.operation(226,{255:room,215:1},reject=True)
            game.disconnect()
            return
        actor = game.operation(226,{255:room,215:1})[254]
        with_peers.pop()
        return game,actor
    finally:
        for peer in with_peers:
            peer.close()


def run(name_port, room, first="probe-a", second="probe-b", app=APP, tickets=(None,None), active=None):
    a, actor_a = enter(name_port,first,room,app,tickets[0])
    try:
        b, actor_b = enter(name_port,second,room,app,tickets[1])
        try:
            assert actor_a != actor_b
            if active: active()
            a.send_operation(253,{244:40,245:b"payload from A"})
            from_a = b.event(40)
            assert from_a[245] == b"payload from A" and from_a[254] == actor_a
            b.send_operation(253,{244:41,245:b"payload from B"})
            from_b = a.event(41)
            assert from_b[245] == b"payload from B" and from_b[254] == actor_b
            b.operation(254,{})
            a.event(254)
        finally:
            b.close()
        b, actor_b = enter(name_port,second,room,app,tickets[1])
        try:
            b.send_operation(253,{244:42,245:b"payload after rejoin"})
            rejoined = a.event(42)
            assert rejoined[245] == b"payload after rejoin" and rejoined[254] == actor_b
            b.disconnect()
            a.event(254)
        finally:
            b.close()
        b, _ = enter(name_port,second,room,app,tickets[1])
        try:
            a.send_operation(253,{244:43,245:b"payload after disconnect"})
            assert b.event(43)[245] == b"payload after disconnect"
        finally:
            b.disconnect()
            b.close()
    finally:
        a.disconnect()
        a.close()
    print("PASS | Luxon discovery and Name → Master → Game, bidirectional events, leave, disconnect and rejoin")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name-port", type=int, default=5058)
    parser.add_argument("--room", default=None)
    args = parser.parse_args()
    run(args.name_port,args.room or "kromarec-probe-" + secrets.token_hex(8))
