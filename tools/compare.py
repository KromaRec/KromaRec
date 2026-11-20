#!/usr/bin/env python3
"""Compare read-only bootstrap contracts against two operator-controlled deployments."""
import argparse
import json
import urllib.error
import urllib.request

CASES = (
    ("api", "/api/versioncheck/v4?v=20230414"),
    ("api", "/api/versioncheck/islandedversions"),
    ("api", "/api/config/v1/amplitude"),
    ("api", "/api/config/v1/backtrace"),
    ("accounts", "/"),
    ("accounts", "/emojiConfig/whitelistedEmojis"),
    ("moderation", "/voice/config"),
    ("playersettings", "/playersettings"),
    ("cdn", "/config/LoadingScreenTipData"),
)


def fetch(domain, service, path):
    url = f"https://{service}.{domain}{path}"
    try:
        response = urllib.request.urlopen(url, timeout=10)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        body = response.read()
        return response.status, json.loads(body) if body else None


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("reference", help="Base domain of a deployment pinned to RecFlare 0.0.10")
parser.add_argument("port", help="Base domain of the KromaRec deployment")
args = parser.parse_args()
failed = False
for service, path in CASES:
    upstream = fetch(args.reference, service, path)
    rewritten = fetch(args.port, service, path)
    equal = upstream == rewritten
    failed |= not equal
    print(f"{'PASS' if equal else 'DIFF'} {service} {path}")
raise SystemExit(int(failed))
