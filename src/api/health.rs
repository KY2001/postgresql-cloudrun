use std::time::Duration;

use openapi::apis::default::HealthResponse;

use crate::postgres;

// GET /health
pub async fn health(open: bool) -> HealthResponse {
    let healthy = tokio::time::timeout(Duration::from_secs(3), postgres::healthy()).await;
    if open && matches!(healthy, Ok(true)) {
        HealthResponse::Status204_TheDatabaseIsResponsiveAndWALArchivingWorks
    } else {
        HealthResponse::Status503_TheDatabaseOrWALArchivingIsUnavailable
    }
}
