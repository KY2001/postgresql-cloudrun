use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Host};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteSqlResponse {
    /// Statements executed
    Status200_StatementsExecuted
    (Vec<models::Result>)
    ,
    /// SQL error
    Status400_SQLError
    (models::ErrorResponse)
    ,
    /// No database connection available
    Status503_NoDatabaseConnectionAvailable
    (models::ErrorResponse)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum HealthResponse {
    /// The database is responsive and WAL archiving works
    Status204_TheDatabaseIsResponsiveAndWALArchivingWorks
    ,
    /// The database or WAL archiving is unavailable
    Status503_TheDatabaseOrWALArchivingIsUnavailable
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum StopResponse {
    /// The database was handed off
    Status204_TheDatabaseWasHandedOff
    ,
    /// The caller is this revision, so nothing was stopped
    Status409_TheCallerIsThisRevision
    ,
    /// Archiving failed
    Status500_ArchivingFailed
    (models::ErrorResponse)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SyncReplicaResponse {
    /// All changes are archived
    Status204_AllChangesAreArchived
    ,
    /// Archiving or backup failed
    Status500_ArchivingOrBackupFailed
    (models::ErrorResponse)
    ,
    /// The database was handed off to a new revision
    Status503_TheDatabaseWasHandedOffToANewRevision
    (models::ErrorResponse)
}




/// Default
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Default<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// ExecuteSql - POST /sql
    async fn execute_sql(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &Vec<models::Statement>,
    ) -> Result<ExecuteSqlResponse, E>;

    /// Check the database and WAL archiving.
    ///
    /// Health - GET /health
    async fn health(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<HealthResponse, E>;

    /// Stop - POST /stop
    async fn stop(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      query_params: &models::StopQueryParams,
    ) -> Result<StopResponse, E>;

    /// Archive WAL to GCS, and take a full backup once an hour.
    ///
    /// SyncReplica - POST /sync
    async fn sync_replica(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<SyncReplicaResponse, E>;
}
