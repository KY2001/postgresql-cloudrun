# Benchmark

Latency, throughput and handoff downtime of the dev service, measured with [bench.py](bench.py).

```sh
URL=https://postgresql-cloudrun-dev-355496381128.asia-northeast1.run.app
benchmark/bench.py latency $URL
benchmark/bench.py throughput $URL
benchmark/bench.py handoff $URL -- make gcloud-deploy   # with .env.dev loaded
```

The server reports its own time in a `Server-Timing` header, and the latencies below come from that header, so they exclude the network round trip (about 15 ms p50 from the client below).

## Setup

- Service: `asia-northeast1`, 1 vCPU, 512 MiB, gen2, max 1 instance, concurrency 1000 ([terraform/environments/dev](../terraform/environments/dev/main.tf))
- Image: PostgreSQL 18.6 and pgBackRest 2.59.2, 1 MB WAL segments ([postgresql.conf](../postgresql.conf), [pgbackrest.conf](../pgbackrest.conf))
- Client: a laptop on a home internet connection, 2026-10-04
- Queries: `SELECT 1`; `read` = `SELECT v FROM bench WHERE id = $1` over 1,000 rows; `write` = `INSERT INTO bench (v) VALUES ($1)` with a 100-byte value; `txn` = `write` and `read` in one request (one transaction)

## Results

### Latency

200 sequential requests per row.

| Request | p50 | p90 | p99 |
| --- | ---: | ---: | ---: |
| `/sql` `SELECT 1` | 0.28 ms | 0.31 ms | 0.37 ms |
| `/sql` read | 0.34 ms | 0.37 ms | 0.41 ms |
| `/sql` write | 0.58 ms | 0.69 ms | 0.75 ms |
| `/sql` txn | 0.75 ms | 0.80 ms | 0.84 ms |
| `GET /health` | 2.68 ms | 2.80 ms | 3.12 ms |
| `POST /sync`, nothing to archive | 3.16 ms | 3.47 ms | 3.80 ms |
| `POST /sync` after a write | 185 ms | 197 ms | 204 ms |

`/health` and `/sync` open a new superuser connection, which costs about 3 ms.
`/sync` after a write waits for `pgbackrest archive-push` to upload one WAL segment, which takes about 165 ms:
a token from the metadata server (~10 ms), `archive.info` (~35 ms with the TLS handshake), a listing to reject a
duplicate segment (~25 ms) and the upload (~75 ms with a second token and TLS handshake).
With WAL-G it took 213 ms, since WAL-G's GCS backend uploads a temporary object, composes it and deletes it.

### Throughput

Each client sends requests back to back for 10 seconds.

| Query | Clients | req/s | p50 | p99 | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| read | 1 | 52 | 0.44 ms | 0.66 ms | 0 |
| read | 10 | 584 | 0.37 ms | 0.64 ms | 0 |
| read | 50 | 559 | 0.51 ms | 24.87 ms | 0 |
| read | 100 | 556 | 3.01 ms | 97.72 ms | 0 |
| write | 1 | 61 | 0.89 ms | 7.15 ms | 0 |
| write | 10 | 654 | 0.69 ms | 1.76 ms | 0 |
| write | 50 | 480 | 1.35 ms | 44.00 ms | 0 |
| write | 100 | 555 | 6.47 ms | 53.63 ms | 0 |

Throughput levels off at 500–650 req/s from 10 clients on, like sqlite-cloudrun from the same client.
Beyond 10 clients requests queue for the 10 pooled connections, which shows in p99.

### Handoff

A single client inserts one row after another while a new revision is deployed.
Downtime runs from the last successful write before the failures to the first successful write after them.
Stop → listening is measured from `stopped the serving revision` to `listening on` in the new revision's logs.

| Run | Downtime | Stop → listening | Acked writes lost |
| --- | ---: | ---: | ---: |
| 1 | 4.55 s | 0.71 s | 0 |
| 2, after a 15 s write burst | 2.05 s | 0.82 s | 0 |

The new revision replays the archived WAL as a standby while the old one still serves, so only the last changes are replayed after `/stop`.
The rest of the downtime is Cloud Run shifting traffic.
