# Back up an existing manual Alpine installation

Run from the live KromaRec working directory as the server user. These commands assume the existing environment is trusted shell syntax. They keep the environment private, use SQLite online backup to include committed WAL state, and save the installed executable. Adapt the executable path for OpenRC. For an update that changes service or proxy configuration, back up those files separately first.

```sh
(
  set +x
  set -eu
  umask 077
  . ./server.env
  db=${KROMAREC_DB:?}
  case "$db" in /*) ;; *) db="$PWD/$db" ;; esac
  test -f "$db"
  mkdir -p "$HOME/kromarec-backups"
  b=$(mktemp -d "$HOME/kromarec-backups/$(date -u +%Y%m%dT%H%M%SZ)-XXXXXX")
  cp server.env "$b/server.env"
  chmod 600 "$b/server.env"
  cmp -s server.env "$b/server.env"
  cp /usr/local/bin/kromarec "$b/kromarec.previous"
  printf '%s\n' "$db" > "$b/database-path"
  cd "$b"
  sqlite3 -readonly "$db" '.timeout 10000' '.backup snapshot.sqlite3'
  test "$(sqlite3 -batch -noheader -readonly snapshot.sqlite3 'PRAGMA integrity_check;')" = ok
  test -z "$(sqlite3 -batch -noheader -readonly snapshot.sqlite3 'PRAGMA foreign_key_check;')"
  sqlite3 -batch -noheader -readonly snapshot.sqlite3 'PRAGMA user_version; SELECT count(*) FROM local_account; SELECT count(*) FROM room;'
  sha256sum server.env snapshot.sqlite3 kromarec.previous database-path > SHA256SUMS
  sha256sum -c SHA256SUMS
  printf 'Verified backup %s\n' "$b"
)
```

Record the printed backup directory and counts. The snapshot is an independent SQLite database. Keep it out of Git.
