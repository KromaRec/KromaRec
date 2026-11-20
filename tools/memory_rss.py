#!/usr/bin/env python3
"""Repeat the local HTTP RSS workload and measure RSS and backend process CPU time."""
import argparse
import os
from pathlib import Path
from statistics import median
from tempfile import TemporaryDirectory
from time import sleep

from smoke import smoke


def cpu_seconds(pid):
    # /proc/PID/stat has a parenthesized command name before the numeric fields.
    stat = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return (int(stat[11]) + int(stat[12])) / os.sysconf("SC_CLK_TCK")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binaries", nargs="+", type=Path)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--schema", type=int, default=8)
    parser.add_argument("--idle-seconds", type=float, default=2.0)
    args = parser.parse_args()
    if args.idle_seconds <= 0:
        parser.error("--idle-seconds must be positive")
    if args.runs < 1:
        parser.error("--runs must be positive")
    for path in args.binaries:
        binary = path.resolve()
        samples = []
        idle_cpu = []
        workload_cpu = []

        def sample(stage, pid):
            nonlocal before_workload
            if stage == "idle":
                before = cpu_seconds(pid)
                sleep(args.idle_seconds)
                before_workload = cpu_seconds(pid)
                idle_cpu.append(before_workload - before)
            else:
                workload_cpu.append(cpu_seconds(pid) - before_workload)

        before_workload = 0.0
        for _ in range(args.runs):
            with TemporaryDirectory() as tmp:
                samples.append(smoke(binary, Path(tmp), args.schema, sample))
        idle, warm = map(median, zip(*samples))
        print(f"{path}: {binary.stat().st_size} bytes, median idle {idle:g} KiB, warm {warm:g} KiB")
        print(f"  RSS samples (idle, 100 open keep-alives): {samples}")
        print(f"  Process CPU medians: idle over {args.idle_seconds:g} s {median(idle_cpu):.3f} s; "
              f"HTTP workload {median(workload_cpu):.3f} s")


if __name__ == "__main__":
    main()
