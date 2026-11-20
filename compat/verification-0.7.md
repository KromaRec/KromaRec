# KromaRec 0.7.0 verification

## What ran

- The interrupted musl executable survived and passed the smoke suite. Fresh
  optimized builds of the current source succeeded for x86-64 musl and glibc
  with Rust 1.100.0 and the locked dependencies.
- Both fresh builds passed the HTTP and room integration suites with patched
  Luxon 1.5. The tests carried forward a genuine 0.6 database and verified
  account, settings, and room persistence, SQLite WAL and integrity, login
  locks, Orientation, dorm and scene metadata, concurrent joins, capacity,
  invites, SignalR, room cleanup, and restart. Schema version remains 6.
- Patched Luxon accepted valid custom-auth tickets in Authenticate and
  AuthenticateOnce and replaced a spoofed user ID. It rejected invalid,
  expired, wrong-app and wrong-purpose credentials, bans, and missing or
  revoked reservations. Two wire peers joined one Game room, exchanged
  events, then exercised leave and rejoin. The regression probe transmits
  credentials on loopback and does not test the game's encrypted path.
- The fresh static musl executable started in an Alpine 3.22 userspace
  chroot, served the 20230414 version endpoint, and shut down on SIGTERM.
  OpenRC service lifecycle and a native Alpine Luxon build were not verified.

## First unverified real-client stage

No 20230414 game installation or generated BepInEx IL2CPP interop assemblies
were available. The first unverified stage is launching that build and
completing manual local-password login. After login, verify the client's
connection-info callback, encrypted NameServer connection, and Harmony
binding for Authenticate or AuthenticateOnce. The optional source overlay
applied cleanly to the pinned RecFlare patch checkout. It was not compiled
against the actual client assemblies. Photon ticket delivery through the
overlay, Unity rendering, player synchronization, gestures, real reconnect
behavior, and Photon Voice are unverified. The [client setup and acceptance
guide](multiplayer.md) gives the steps and expected evidence.

## Memory snapshots

Three sequential smoke runs per executable on Debian 13, reported as median
RSS in KiB. Warm samples use 100 HTTP keep-alives after the smoke workload.

| Build | Binary bytes | Idle RSS | Warm RSS |
| --- | ---: | ---: | ---: |
| 0.6 static musl baseline | 3,394,976 | 3,304 | 6,072 |
| 0.7 static musl | 3,394,976 | 3,412 | 6,112 |
| 0.7 glibc | 3,265,544 | 4,128 | 25,660 |

The static build's median deltas were +108 KiB idle and +40 KiB warm,
within the spread observed across the runs.

| Release build with two wire peers admitted | KromaRec RSS | Luxon RSS | Combined RSS |
| --- | ---: | ---: | ---: |
| Static musl KromaRec | 3,624 KiB | 3,724 KiB | 7,348 KiB |
| glibc KromaRec | 23,716 KiB | 3,656 KiB | 27,372 KiB |

The static build met the requested RSS and binary size targets in these
workloads. The glibc build exceeded the warm and combined targets.
These are snapshots, not peak usage. The combined column sums process RSS
and can count shared pages twice. TLS proxy and game-client RAM are excluded.

To repeat, use `tools/memory_rss.py` for three idle/warm pairs per binary.
Use `tools/rooms_smoke.py --secure-luxon ...` for the two-peer snapshots.
Both programs use Linux procfs.
