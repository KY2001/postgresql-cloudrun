# syntax=docker/dockerfile:1
# check=error=true

# For building the binary
FROM rust:1.99.0-slim-trixie AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY openapi ./openapi
COPY src ./src
RUN cargo build --release --locked

# For deployment
FROM postgres:18.6-trixie AS deploy
RUN apt-get update \
    && apt-get install -y --no-install-recommends pgbackrest=2.59.2-1.pgdg13+1 ca-certificates=20250419 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/target/release/postgresql-cloudrun /usr/local/bin/
COPY postgresql.conf /etc/postgresql/
COPY pgbackrest.conf /etc/pgbackrest/
RUN install --directory --owner=postgres /data
ENV PGDATA=/data/pgdata PGHOST=/tmp PGUSER=postgres
# The postgres user; PostgreSQL refuses to run as root.
USER 999
# The server stops PostgreSQL on SIGTERM, which Cloud Run sends; the base image uses SIGINT.
STOPSIGNAL SIGTERM

ENTRYPOINT ["/usr/local/bin/postgresql-cloudrun"]
