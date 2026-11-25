#!/usr/bin/env python3
"""Refresh reference locations without discarding maintained implementation decisions."""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def rows(name):
    with (ROOT / name).open() as src:
        return list(csv.DictReader(src, delimiter="\t"))


def key(row):
    # Mounts can repeat at the same service and path within one reference file.
    return tuple(row[k] for k in ("service", "method", "path", "source", "line"))


if __name__ == "__main__":
    current = rows("route-status.tsv")
    upstream = rows("upstream-routes.tsv")
    by_key = {key(r): r for r in current}
    keys = [key(r) for r in upstream]
    if len(by_key) != len(current) or len(set(keys)) != len(keys):
        raise SystemExit("duplicate reference row")
    if by_key.keys() - set(keys):
        raise SystemExit("upstream changed; review moved or removed rows before refreshing")
    target = ROOT / "route-status.tsv"
    temp = target.with_suffix(".tsv.tmp")
    with temp.open("w") as out:
        writer = csv.DictWriter(out, fieldnames=[*upstream[0], "status", "note"],
                                delimiter="\t", lineterminator="\n")
        writer.writeheader()
        for r in upstream:
            writer.writerow(by_key.get(key(r), {**r, "status":"not yet implemented", "note":""}))
    temp.replace(target)
