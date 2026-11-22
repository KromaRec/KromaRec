#!/usr/bin/env python3
"""Generate route status from the upstream inventory and the current port coverage."""
import csv
from pathlib import Path

HERE = Path(__file__).resolve().parent
IMPLEMENTED = {
    "ns": {"GET /"},
    "discovery": {"GET /"},
    "api": {"GET /api/config/v1/amplitude", "GET /api/config/v1/azurespeech",
            "GET /api/config/v1/backtrace", "GET /api/config/v2", "GET /api/gameconfigs/v1/all",
            "GET /api/versioncheck/v4", "GET /api/versioncheck/islandedversions",
            "POST /statsigUserProperties", "GET /voice/config"},
    "accounts": {"GET /", "GET /emojiConfig/whitelistedEmojis", "GET /account/me",
                 "GET /account/:id", "GET /account/username/:name", "POST /account/create"},
    "auth": {"POST /connect/token"},
    "playersettings": {"GET /", "GET /playersettings", "PUT /playersettings", "DELETE /playersettings"},
    "moderation": {"GET /", "GET /voice/config"},
    "cdn": {"GET /", "GET /config/LoadingScreenTipData", "GET /config/:name", "GET /sigs/:sigName",
            "GET /room/:dataBlob{.+}", "GET /invention/:dataBlob{.+}", "GET /data/:id{.+}",
            "GET /avatar/:asset{.+}", "GET /roomunityasset/:filename{.+}"},
}
MILESTONE = {
    "auth": {"POST /account/me/changepassword"},
    "accounts": {"GET /account/:id/bio", "PUT /account/me/bio",
                 "POST /account/me/identityflags", "PUT /account/me/personalpronouns"},
    "match": {"POST /player/login", "POST /player/logout", "POST /player/heartbeat"},
    "api": {"GET /api/relationships/v2/get", "GET /api/relationships/mutualfriends",
            "GET /api/messages/v2/get", "POST /api/messages/v3/delete",
            "POST /api/messages/v1/friendOnlineStatus"} |
           {f"{method} /api/relationships/{version}/{action}"
            for method in ("GET", "POST")
            for version, actions in (
                ("v2", ("sendfriendrequest", "acceptfriendrequest", "removefriend", "addfriend")),
                ("v1", ("ignore", "unignore", "mute", "unmute", "favorite", "unfavorite")),
            )
            for action in actions},
}
with (HERE / "upstream-routes.tsv").open() as source, (HERE / "route-status.tsv").open("w") as target:
    rows = csv.DictReader(source, delimiter="\t")
    writer = csv.DictWriter(target, fieldnames=[*rows.fieldnames, "status", "note"],
                            delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for row in rows:
        service, method, path = (row[key] for key in ("service", "method", "path"))
        status, note = "not yet implemented", ""
        if f"{method} {path}" in IMPLEMENTED.get(service, set()):
            status = "implemented but awaiting client verification"
        if service == "datacollection":
            note = "Excluded by the no-telemetry requirement"
        if service == "auth" and path == "/connect/token":
            status, note = "not yet implemented", "Password signup, login and refresh are implemented. Platform, Studio and other grants pending."
        if service == "accounts" and path == "/account/create":
            note = "Requires explicit signup opt-in"
        if service == "api" and path == "/statsigUserProperties":
            note = "Returns disabled for the no-telemetry requirement"
        if f"{method} {path}" in MILESTONE.get(service, set()):
            status = "implemented but awaiting client verification"
            note = "Covered by local HTTP smoke tests"
            if service == "match" or path.endswith("/friendOnlineStatus"):
                note += "; lobby only, no room presence or push notifications"
            if path.startswith("/api/relationships/") or service == "accounts":
                note += "; real-time notifications pending"
        writer.writerow({**row, "status": status, "note": note})
