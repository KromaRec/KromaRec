# Upgrade and roll back

The 0.10.2 → 0.10.3 upgrade uses the same schema 9. Back up the live environment, database and executable as described below. For sites upgrading from 0.10.1, version 0.10.2 introduced four read-model tables and a room-comment index through migration 009. The SQLite transaction advances schema 8 to 9. Take a consistent backup after quiescing writers and before the first start. The backend applies supported older migrations on open. Test an older production snapshot in staging first.

1. Extract into a fresh staging directory. Check SHA256SUMS. Follow BACKUP.md in the live directory and record the backup path. Keep the live environment, assets, Caddy configuration and database in place.
2. In staging run `python3 tools/smoke.py ./bin/kromarec` in a clean shell with no production variables exported. This uses a temporary database.
3. For a maintenance window stop Luxon if running, then KromaRec, with their existing service manager or Ctrl+C for a foreground launch. Confirm no backend owns 8080 or 8081.
4. From staging, run `doas install -m 0755 bin/kromarec /usr/local/bin/kromarec` for the manual Alpine deployment. OpenRC users should install to their configured executable path. Start the backend from its original working directory with the original environment, then start the already configured patched Luxon. Verify both loopback bindings, the HTTPS service directory and a previously existing room. No proxy restart is required for a binary-only update.
5. To roll back this update, stop Luxon and KromaRec, set `b` to the verified backup directory, run `(cd "$b" && sha256sum -c SHA256SUMS)`, then `doas install -m 0755 "$b/kromarec.previous" /usr/local/bin/kromarec`. For rollback to 0.10.2, retain the current schema 9 database and restart the previous backend followed by Luxon. Before starting 0.10.1, restore the verified pre-upgrade schema 8 snapshot using the procedure below. Schema 9 cannot be opened by 0.10.1. Save the stopped schema 9 database and its sidecars so writes since upgrade are recoverable separately.

## Restore a pre-upgrade snapshot during rollback or database recovery

Restoring a snapshot discards changes committed after backup. Stop every writer first. From the original working directory, with `b` set and checksums verified, use the following as the database owner. It moves the stopped database and its sidecars aside together and restores into the original location. Ensure adequate disk space. Restore server.env from the verified backup only if its configuration also requires recovery.

```sh
(
  set -eu
  umask 077
  db=$(cat "$b/database-path")
  salvage=$(mktemp -d "$(dirname "$db")/kromarec-recovery-XXXXXX")
  for f in "$db" "$db-wal" "$db-shm" "$db-journal"; do
    if test -e "$f"; then mv "$f" "$salvage/"; fi
  done
  cp "$b/snapshot.sqlite3" "$db"
  test "$(sqlite3 -batch -noheader -readonly "$db" 'PRAGMA integrity_check;')" = ok
  test -z "$(sqlite3 -batch -noheader -readonly "$db" 'PRAGMA foreign_key_check;')"
)
```

Verify ownership, permissions, account and room counts before resuming the backend and Luxon.
