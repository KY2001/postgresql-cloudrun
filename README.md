# postgresql-cloudrun

An HTTP server that runs raw SQL against a PostgreSQL database on Cloud Run.
PostgreSQL runs inside the container, and [pgBackRest](https://pgbackrest.org) backs it up to Google Cloud Storage and restores it on startup.
It is the PostgreSQL version of [sqlite-cloudrun](https://github.com/KY2001/sqlite-cloudrun).

## Highlights

- **Affordable**: request-based billing and scale to zero, without Cloud SQL. You pay only while requests are running, plus GCS storage.
- **Simple**: plain SQL over HTTP, JSON in and out.
- **Close to your users**: deploy to any Google Cloud region.

## Endpoints

| Endpoint | Description |
| --- | --- |
| `POST /sql` | Runs statements in order and returns one result per statement. Two or more statements run in one transaction. |
| `POST /sync` | Waits until all changes are archived to GCS, and takes a full backup once an hour. |
| `POST /stop` | Hands the database off to a new revision. Called by the new revision on startup. |
| `GET /health` | Checks the database and WAL archiving. |

```sh
curl -X POST $URL/sql -H 'Content-Type: application/json' \
  -d '[{"sql": "SELECT $1::int AS x", "params": [1]}]'
# [{"columns":["x"],"types":["int4"],"rows":[[1]],"changes":1}]

curl -X POST $URL/sync   # 204 once all changes are in GCS
curl $URL/health         # 204 if the database is responsive and WAL archiving works
```

See [openapi/openapi.yaml](openapi/openapi.yaml) for the full API.

## FAQ

### How are parameters and results typed?
Parameters are sent as text, and PostgreSQL converts them to the placeholder's type (arrays and objects are sent as JSON).
An untyped placeholder such as `SELECT $1` is `text`; cast it (`$1::int`) to get another type.
In results, booleans, integers and finite floats are JSON values (NaN and infinities are the strings `NaN`, `Infinity` and `-Infinity`), `json`/`jsonb` is JSON, `bytea` is a hex string, and the rest (including `numeric`) are strings. Cast arrays and other unsupported types to `text`.

### How are transactions handled?
A request with one statement runs in autocommit mode.
A request with two or more statements runs in a single transaction: all succeed or all are rolled back.
`BEGIN`, `COMMIT`, `ROLLBACK` and the like are not allowed, and a transaction can't span requests.
Each statement times out after 5 seconds.

### Who runs the SQL?
The `app` role, which owns the `app` database but is not a superuser, so SQL can't read files or run programs in the container.

### Is it consistent?
Yes. The service runs on a single instance (`--max-instances=1`), so every request sees the latest committed data.

### Can I lose data?
Basically No. On a normal shutdown, Cloud Run sends `SIGTERM` and the server archives the remaining WAL to GCS before exiting.
If the instance crashes, writes since the last archived WAL segment are lost. With request-based billing PostgreSQL gets CPU only while requests run, so `archive_timeout` alone can't be relied on; a Cloud Scheduler job calls `/sync` every minute, which bounds the loss to about a minute. Call `/sync` after a write to make it durable right away.

### Are there cold starts?
Rarely. A Cloud Scheduler job calls `/sync` every minute, which keeps the instance warm.

### What happens on deploy?
The new revision takes the database over before it starts serving. It restores the latest full backup and replays the archived WAL as a standby while the old revision still serves. It then calls `POST /stop`, which Cloud Run routes to the old revision; the old revision finishes in-flight queries, archives the remaining WAL and shuts PostgreSQL down. The new revision replays that WAL, is promoted, and starts serving.
Requests during the handoff (a few seconds) get `503`; clients should retry.

## Benchmark

See [benchmark](benchmark/README.md).
