#!/usr/bin/env python3
"""Measures latency, throughput and handoff downtime of a deployed postgresql-cloudrun service.

  bench.py latency URL                 # sequential requests, one at a time
  bench.py throughput URL              # concurrent requests for a fixed duration
  bench.py handoff URL -- COMMAND...   # writes continuously while COMMAND deploys a new revision

Latencies are the server's own time from the Server-Timing header, so they exclude the network.
Uses only the standard library. Drops and recreates a table named `bench`.
"""

import argparse
import http.client
import json
import random
import statistics
import subprocess
import threading
import time
import urllib.parse

ROWS = 1000
READ = lambda: [{"sql": "SELECT v FROM bench WHERE id = $1", "params": [random.randint(1, ROWS)]}]
WRITE = lambda: [{"sql": "INSERT INTO bench (v) VALUES ($1)", "params": ["x" * 100]}]
# Two statements run in one transaction.
TXN = lambda: WRITE() + READ()
NOOP = lambda: [{"sql": "SELECT 1"}]


class Client:
    """One keep-alive connection that reconnects after an error."""

    def __init__(self, url):
        self.url = urllib.parse.urlsplit(url)
        self.conn = None

    def sql(self, statements):
        return self.request("POST", "/sql", json.dumps(statements))

    def request(self, method, path, body=None):
        """Returns (HTTP status or 0 on a connection error, server time in seconds, response body)."""
        if self.conn is None:
            cls = http.client.HTTPSConnection if self.url.scheme == "https" else http.client.HTTPConnection
            self.conn = cls(self.url.netloc, timeout=10)
        try:
            self.conn.request(method, path, body, {"Content-Type": "application/json"})
            response = self.conn.getresponse()
            body = response.read()
        except (OSError, http.client.HTTPException):
            self.conn.close()
            self.conn = None
            return 0, float("nan"), b""
        # Server-Timing: app;dur=<milliseconds>
        timing = response.getheader("Server-Timing", "dur=nan")
        return response.status, float(timing.split("dur=")[1]) / 1000, body


def setup(url):
    status, _, body = Client(url).sql([
        {"sql": "DROP TABLE IF EXISTS bench"},
        {"sql": "CREATE TABLE bench (id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY, v text NOT NULL)"},
        {"sql": f"INSERT INTO bench (v) SELECT left(repeat(md5(random()::text), 4), 100) "
                f"FROM generate_series(1, {ROWS})"},
    ])
    assert status == 200, (status, body)


def ms(seconds):
    return f"{seconds * 1000:7.2f}"


def percentiles(latencies):
    q = statistics.quantiles(latencies, n=100, method="inclusive")
    return q[49], q[89], q[98]


def latency(url, n):
    client = Client(url)
    requests = [
        ("SELECT 1", lambda: client.sql(NOOP())),
        ("read", lambda: client.sql(READ())),
        ("write", lambda: client.sql(WRITE())),
        ("txn", lambda: client.sql(TXN())),
        ("/health", lambda: client.request("GET", "/health")),
        ("/sync", lambda: client.request("POST", "/sync")),
    ]
    print(f"{'request':<8} {'p50 ms':>7} {'p90 ms':>7} {'p99 ms':>7}")
    for name, request in requests:
        for _ in range(10):  # Warm up the connection.
            request()
        results = [request() for _ in range(n)]
        assert all(status in (200, 204) for status, _, _ in results), name
        p50, p90, p99 = percentiles([seconds for _, seconds, _ in results])
        print(f"{name:<8} {ms(p50)} {ms(p90)} {ms(p99)}")


def load(url, statements, concurrency, duration):
    """Runs `concurrency` clients for `duration` seconds and returns [(status, server time)]."""
    results = []
    ready = threading.Barrier(concurrency + 1)
    deadline = [0.0]

    def worker():
        client = Client(url)
        client.sql(NOOP())  # Open the connection before the clock starts.
        ready.wait()
        while time.perf_counter() < deadline[0]:
            status, seconds, _ = client.sql(statements())
            results.append((status, seconds))

    threads = [threading.Thread(target=worker) for _ in range(concurrency)]
    for thread in threads:
        thread.start()
    deadline[0] = time.perf_counter() + duration
    ready.wait()
    for thread in threads:
        thread.join()
    return results


def throughput(url, duration, concurrencies):
    print(f"{'query':<6} {'clients':>7} {'req/s':>7} {'p50 ms':>7} {'p99 ms':>7} {'errors':>6}")
    for name, statements in [("read", READ), ("write", WRITE)]:
        for concurrency in concurrencies:
            results = load(url, statements, concurrency, duration)
            ok = [seconds for status, seconds in results if status == 200]
            p50, _, p99 = percentiles(ok)
            print(f"{name:<6} {concurrency:>7} {len(ok) / duration:>7.0f} {ms(p50)} {ms(p99)} "
                  f"{len(results) - len(ok):>6}")


def handoff(url, command):
    """Writes one row at a time while `command` runs, then reports downtime and lost writes."""
    stop = threading.Event()
    log = []  # (start time, status)

    def writer():
        client = Client(url)
        while not stop.is_set():
            start = time.monotonic()
            status, _, _ = client.sql([{"sql": "INSERT INTO bench (v) VALUES ('handoff')"}])
            log.append((start, status))
            if status != 200:
                time.sleep(0.05)

    thread = threading.Thread(target=writer)
    thread.start()
    start = time.monotonic()
    subprocess.run(command, check=True)
    deployed = time.monotonic() - start
    time.sleep(10)  # Let traffic settle on the new revision.
    stop.set()
    thread.join()

    # A window runs from the last success before a failure to the first success after it.
    windows, last_ok, failing = [], None, False
    for t, status in log:
        if status == 200:
            if failing and last_ok is not None:
                windows.append((last_ok - start, t - last_ok))
            last_ok, failing = t, False
        else:
            failing = True
    acked = sum(status == 200 for _, status in log)
    _, _, body = Client(url).sql([{"sql": "SELECT count(*) FROM bench WHERE v = 'handoff'"}])
    stored = json.loads(body)[0]["rows"][0][0]

    print(f"deploy command:  {deployed:.1f} s")
    print(f"requests:        {len(log)} ({len(log) - acked} failed)")
    for at, length in windows:
        print(f"downtime:        {length:.2f} s (at +{at:.1f} s)")
    print(f"acked writes:    {acked}, stored: {stored}, lost: {max(acked - stored, 0)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="mode", required=True)
    p = sub.add_parser("latency")
    p.add_argument("url")
    p.add_argument("-n", type=int, default=200, help="requests per query")
    p = sub.add_parser("throughput")
    p.add_argument("url")
    p.add_argument("-d", "--duration", type=float, default=10, help="seconds per run")
    p.add_argument("-c", "--concurrency", type=int, nargs="+", default=[1, 10, 50, 100])
    p = sub.add_parser("handoff")
    p.add_argument("url")
    p.add_argument("command", nargs="+", help="deploys a new revision, e.g. make gcloud-deploy")
    args = parser.parse_args()

    setup(args.url)
    if args.mode == "latency":
        latency(args.url, args.n)
    elif args.mode == "throughput":
        throughput(args.url, args.duration, args.concurrency)
    else:
        handoff(args.url, args.command)


if __name__ == "__main__":
    main()
