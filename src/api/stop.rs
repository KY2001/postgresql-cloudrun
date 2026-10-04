use openapi::{apis::default::StopResponse, models::ErrorResponse};

use super::Server;

// POST /stop
pub async fn stop(server: &Server, revision: &str) -> StopResponse {
    // It must not stop itself.
    if std::env::var("K_REVISION").is_ok_and(|own| own == revision) {
        return StopResponse::Status409_TheCallerIsThisRevision;
    }
    match server.stop().await {
        Ok(()) => StopResponse::Status204_TheDatabaseWasHandedOff,
        Err(e) => StopResponse::Status500_ArchivingFailed(ErrorResponse::new(
            500,
            format!("archive: {e}"),
        )),
    }
}
