use std::{
    future::Future,
    time::{Duration, Instant},
};

use tokio::process::Command;
use tokio_postgres::{Client, NoTls};

const CONFIG: &str = "-c config_file=/etc/postgresql/postgresql.conf";
const SUPERUSER: &str = "host=/tmp user=postgres dbname=postgres";
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const SYNC_TIMEOUT: Duration = Duration::from_secs(30);

// Restores the latest full backup and replays the archived WAL as a standby while the old revision
// still serves, then calls `stop` and promotes once the WAL it archived on stop is replayed.
// Creates a new cluster if nothing is backed up yet. Returns whether it restored.
pub async fn start(stop: impl Future<Output = ()>) -> bool {
    let output = Command::new("pgbackrest")
        // Keep log lines out of the JSON on stdout.
        .args([
            "--stanza=app",
            "--output=json",
            "--log-level-console=off",
            "info",
        ])
        .output()
        .await
        .expect("run pgbackrest info");
    assert!(output.status.success(), "pgbackrest info: {output:?}");
    let info: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let status = &info[0]["status"];
    match status["code"].as_i64() {
        Some(0) => {}
        // Missing stanza path: nothing was ever backed up.
        Some(1) => {
            stop.await;
            create().await;
            return false;
        }
        _ => panic!("pgbackrest info: {status}"),
    }

    // Writes standby.signal and the restore_command.
    run("pgbackrest", &["--stanza=app", "--type=standby", "restore"])
        .await
        .unwrap();
    // Returns once the standby is consistent; it then keeps replaying newly archived WAL.
    run(
        "pg_ctl",
        &["start", "--wait", "--timeout=3600", "--options", CONFIG],
    )
    .await
    .unwrap();
    // Replay what is archived while the old revision still serves, so only its last changes are
    // left to replay after it stops.
    catch_up().await;

    stop.await;

    // Promotion first replays the WAL already in the archive.
    run("pg_ctl", &["promote", "--wait", "--timeout=3600"])
        .await
        .unwrap();
    true
}

async fn create() {
    // The builtin locale provider doesn't depend on glibc, so a newer image can't corrupt indexes.
    let args = [
        "--locale-provider=builtin",
        "--locale=C.UTF-8",
        "--wal-segsize=1",
        "--auth=trust",
        "--username=postgres",
    ];
    run("initdb", &args).await.unwrap();
    run("pg_ctl", &["start", "--wait", "--options", CONFIG])
        .await
        .unwrap();
    // The stanza holds the backups and archived WAL of this cluster; archiving fails until it exists.
    run("pgbackrest", &["--stanza=app", "stanza-create"])
        .await
        .unwrap();
    let client = connect().await.unwrap();
    // Requests run as `app`, which can't reach the file system or run programs.
    client.batch_execute("CREATE ROLE app LOGIN").await.unwrap();
    client
        .batch_execute("CREATE DATABASE app OWNER app")
        .await
        .unwrap();
    // A restore starts from a full backup.
    backup().await.unwrap();
}

// Waits until the standby has replayed all archived WAL: it then sleeps before asking for the
// next segment again.
async fn catch_up() {
    let client = connect().await.unwrap();
    let caught_up = "SELECT EXISTS (SELECT FROM pg_stat_activity
        WHERE backend_type = 'startup' AND wait_event = 'RecoveryRetrieveRetryInterval')";
    while !client
        .query_one(caught_up, &[])
        .await
        .unwrap()
        .get::<_, bool>(0)
    {
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

pub async fn stop() -> Result<(), String> {
    run("pg_ctl", &["stop", "--wait", "--mode", "fast"]).await
}

// Switches to a new WAL segment and waits until the finished one is archived.
pub async fn sync() -> Result<(), String> {
    let client = connect().await?;
    // pg_switch_wal() returns the end of the finished segment, or the start of the current one if
    // nothing was written since the last switch. Either way the byte before it is in the last
    // segment that needs archiving.
    let wal: String = client
        .query_one("SELECT pg_walfile_name(pg_switch_wal() - 1)", &[])
        .await
        .map_err(|e| e.to_string())?
        .get(0);
    let deadline = Instant::now() + SYNC_TIMEOUT;
    loop {
        let row = client
            .query_one(
                "SELECT coalesce(last_archived_wal >= $1, false), last_failed_wal FROM pg_stat_archiver",
                &[&wal],
            )
            .await
            .map_err(|e| e.to_string())?;
        if row.get(0) {
            return Ok(());
        }
        if Instant::now() > deadline {
            let failed: Option<String> = row.get(1);
            return Err(format!("{wal} not archived; last failed: {failed:?}"));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

// Takes a full backup. pgBackRest then expires all but the last two (repo1-retention-full), with the
// WAL they no longer need.
pub async fn backup() -> Result<(), String> {
    // After a promotion pg_control gets the new timeline only at the next checkpoint, and pgBackRest
    // refuses to back up until it does.
    connect()
        .await?
        .batch_execute("CHECKPOINT")
        .await
        .map_err(|e| e.to_string())?;
    run("pgbackrest", &["--stanza=app", "--type=full", "backup"]).await
}

// Healthy if PostgreSQL answers and the last WAL archiving attempt succeeded.
pub async fn healthy() -> bool {
    let Ok(client) = connect().await else {
        return false;
    };
    let row = client
        .query_one(
            "SELECT last_failed_time IS NULL OR last_failed_time < last_archived_time FROM pg_stat_archiver",
            &[],
        )
        .await;
    matches!(row, Ok(row) if row.get::<_, bool>(0))
}

async fn connect() -> Result<Client, String> {
    let (client, connection) = tokio_postgres::connect(SUPERUSER, NoTls)
        .await
        .map_err(|e| e.to_string())?;
    tokio::spawn(connection);
    Ok(client)
}

async fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .await
        .map_err(|e| format!("{program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} {}: {status}", args.join(" ")))
    }
}
