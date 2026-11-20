#!/usr/bin/env python3
"""Measure discovery requests with concurrent HTTP/1.1 connections on loopback."""
import argparse
import concurrent.futures
import http.client
import statistics
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--port", type=int, default=8080)
parser.add_argument("--host", default="rec.test")
parser.add_argument("--requests", type=int, default=20, help="Requests per connection")
args = parser.parse_args()


def run_worker(barrier):
    times = []
    with_connection = http.client.HTTPConnection("127.0.0.1", args.port, timeout=20)
    try:
        barrier.wait(timeout=20)
        for _ in range(args.requests):
            start = time.perf_counter()
            with_connection.request("GET", "/", headers={"Host": args.host})
            response = with_connection.getresponse()
            response.read()
            if response.status != 200:
                raise RuntimeError(f"Unexpected HTTP status {response.status}")
            times.append((time.perf_counter() - start) * 1000)
    finally:
        with_connection.close()
    return times


for clients in (1, 10, 50, 100, 500):
    barrier = threading.Barrier(clients)
    start = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(clients) as pool:
        timings = [value for result in pool.map(run_worker, [barrier] * clients) for value in result]
    elapsed = time.perf_counter() - start
    p95 = statistics.quantiles(timings, n=100)[94]
    print(f"{clients:3} connections | {len(timings)/elapsed:8.1f} req/s | p95 {p95:7.2f} ms")
