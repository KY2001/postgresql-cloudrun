use std::time::{Duration, Instant};

use openapi::{apis::default::SyncReplicaResponse, models::ErrorResponse};

use super::Server;
use crate::postgres;

// A restore replays the WAL archived since the full backup, so a recent one keeps it fast.
const BACKUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

// POST /sync
pub async fn sync_replica(server: &Server, open: bool) -> SyncReplicaResponse {
    if !open {
        return SyncReplicaResponse::Status503_TheDatabaseWasHandedOffToANewRevision(
            ErrorResponse::new(503, "the database was handed off to a new revision".into()),
        );
    }
    if let Err(e) = postgres::sync().await {
        return failed(format!("archive: {e}"));
    }
    let mut last_backup = server.last_backup.lock().await;
    if last_backup.is_none_or(|last| last.elapsed() > BACKUP_INTERVAL) {
        if let Err(e) = postgres::backup().await {
            return failed(format!("backup: {e}"));
        }
        *last_backup = Some(Instant::now());
    }
    SyncReplicaResponse::Status204_AllChangesAreArchived
}

fn failed(message: String) -> SyncReplicaResponse {
    SyncReplicaResponse::Status500_ArchivingOrBackupFailed(ErrorResponse::new(500, message))
}
