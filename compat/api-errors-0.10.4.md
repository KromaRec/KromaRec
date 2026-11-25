# Browser errors and scene metadata

An anonymous browser-equivalent GET used a disposable configured backend. The companion service matrix records all root bodies through Caddy and direct HTTP. None returned the literal JSON `{"success":"Error"}`. The deployment URL, method and response producing that particular observation remain unknown.

| Host and request | Status | Body | Interpretation |
|---|---:|---|---|
| api GET / | 404 | `{"success":false,"error":{"message":"not found"}}` | No root registration. Version and configuration paths exist. |
| accounts GET /account/me | 401 | Empty | Private self profile needs a current bearer. |
| match GET / | 401 | Empty | The session dispatcher authenticates before route selection. This response alone cannot establish a route exists. |
| auth POST /connect/token without form MIME | 415 | Empty | Rejected before credentials are processed. |
| rooms GET /rooms/13 with DB path only, 0.10.3 | 503 | Empty | DB subsystem disabled because the signing secret was absent, even though a database path was configured. Discovery still returned 200. |
| api POST /api/PlayerReporting/v1/deviceId | 404 | `{"success":false,"error":{"message":"not found"}}` | No evidence of a successful native response contract. |
| rooms GET /rooms/13, configured 0.10.4 | 200 | Room metadata, RoomId 13, subroom 23 | Embedded Orientation scene c79709d8-a31b-48aa-9eb8-cc31ba9505e8. The renderer runs on the client. |

A signing secret with an explicit DB path now enables the persistent service, and a missing secret stops startup before the DB opens. Diagnostic-only startup without both settings remains available and identifies its disabled database in the banner. A missing room seed, incorrect database or unavailable asset must be investigated separately. The settings of the historical failing deployment were unavailable, so its original cause remains open.

To check an existing running installation, run the following read-only requests. Use the actual trusted CA and service names. Run the checks before making configuration changes.

```sh
curl -sS -i -H 'Host: rooms.rec.test' http://127.0.0.1:8080/rooms/13
curl -sS -i --cacert "$PUBLIC_CA" https://rooms.rec.test/rooms/13
```

Compare status, Content-Type, RoomId, SubRoomId and UnitySceneId. For a direct 503, check the running process's startup banner and whether the secret and expected database path were loaded. Never dump the environment. If only HTTPS fails, investigate DNS, proxy Host forwarding and certificate trust. With an authorized account the rooms suite checks private dorm allocation independently of public Orientation lookup. Absent CDN objects continue to return 404.

The route inventory contains the exact error bodies for anonymous probes. Malformed requests can be rejected by a service dispatcher, content-type check or auth guard before a database read. No unexpected 5xx appeared in the configured inventory sweep. No response body, raw query or credential is needed in application logs; existing opt-in startup tracing uses fixed labels.
