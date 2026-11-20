#!/usr/bin/env python3
"""Check trusted HTTPS discovery and bootstrap from a client-side machine."""
import argparse
import json
import ssl
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPSHandler, HTTPRedirectHandler, Request, build_opener


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def https_origin(value):
    parsed = urlsplit(value)
    if (parsed.scheme != "https" or not parsed.hostname or parsed.username
            or parsed.password or parsed.path not in ("", "/")
            or parsed.query or parsed.fragment):
        raise ValueError("expected a bare HTTPS origin")
    return value.rstrip("/")


def verify(nameserver, match, ca=None):
    opener = build_opener(NoRedirect(), HTTPSHandler(context=ssl.create_default_context(cafile=ca)))

    def fetch(origin, route, label, method="GET"):
        try:
            with opener.open(Request(origin + route, method=method), timeout=10) as result:
                if result.status != 200 or result.headers.get_content_type() != "application/json":
                    raise ValueError(f"{label}: expected HTTP 200 application/json")
                body = result.read(2 * 1024 * 1024 + 1)
                if len(body) > 2 * 1024 * 1024:
                    raise ValueError(f"{label}: response exceeds 2 MiB")
                data = json.loads(body)
        except HTTPError as exc:
            raise ValueError(f"{label}: HTTP {exc.code}") from None
        except (URLError, TimeoutError, UnicodeError, json.JSONDecodeError) as exc:
            raise ValueError(f"{label}: {type(exc).__name__}") from None
        print(f"PASS | {label}")
        return data

    discovery = fetch(https_origin(nameserver), "/", "HTTPS discovery")
    labels = ("API", "Auth", "Accounts", "Matchmaking", "Rooms", "Notifications", "PlayerSettings", "CDN")
    if not isinstance(discovery, dict) or not all(isinstance(discovery.get(key), str) for key in labels):
        raise ValueError("discovery: missing required service origin")
    origins = {key: https_origin(discovery[key]) for key in labels}
    if origins["Matchmaking"] != https_origin(match):
        raise ValueError("discovery: Matchmaking differs from the Phase 1 MatchUrl")
    status = fetch(origins["API"], "/api/versioncheck/v4?v=20230414", "versioncheck")
    if not isinstance(status, dict) or status.get("VersionStatus") != 0:
        raise ValueError("versioncheck: server does not accept 20230414")
    fetch(origins["API"], "/api/config/v2", "initial config")
    fetch(origins["API"], "/api/gameconfigs/v1/all", "game configs")
    fetch(origins["Accounts"], "/emojiConfig/whitelistedEmojis", "emoji config")
    fetch(origins["CDN"], "/config/LoadingScreenTipData", "CDN loading tips")
    room = fetch(origins["Rooms"], "/rooms/13", "Orientation room")
    if (not isinstance(room, dict) or room.get("RoomId") != 13
            or not isinstance(room.get("SubRooms"), list)
            or not any(isinstance(subroom, dict) and subroom.get("UnitySceneId")
                       == "c79709d8-a31b-48aa-9eb8-cc31ba9505e8"
                       for subroom in room.get("SubRooms", []))):
        raise ValueError("Orientation room: expected the 20230414 scene")
    negotiation = fetch(origins["Notifications"], "/hub/v1/negotiate?negotiateVersion=1",
                        "SignalR negotiation", method="POST")
    if (not isinstance(negotiation, dict) or negotiation.get("negotiateVersion") != 1
            or not isinstance(negotiation.get("connectionToken"), str)
            or not negotiation["connectionToken"]
            or not isinstance(negotiation.get("availableTransports"), list)
            or not any(isinstance(transport, dict) and transport.get("transport") == "WebSockets"
                       and isinstance(transport.get("transferFormats"), list)
                       and "Text" in transport["transferFormats"]
                       for transport in negotiation.get("availableTransports", []))):
        raise ValueError("SignalR negotiation: expected a text WebSocket transport")



if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("nameserver", help="Phase 1 Backend.BaseUrl")
    parser.add_argument("--match-url", required=True, help="Phase 1 Backend.MatchUrl")
    parser.add_argument("--ca", help="optional trusted CA PEM used for the preflight")
    args = parser.parse_args()
    try:
        verify(args.nameserver, args.match_url, args.ca)
    except ValueError as exc:
        parser.exit(1, f"FAIL | {exc}\n")
