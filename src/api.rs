mod execute_sql;
mod health;
mod stop;
mod sync_replica;

use std::time::Instant;

use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use deadpool_postgres::Pool;
use openapi::apis::{
    default::{Default, ExecuteSqlResponse, HealthResponse, StopResponse, SyncReplicaResponse},
    ErrorHandler,
};
use openapi::models::{Statement, StopQueryParams};
use tokio::sync::{Mutex, RwLock};

use crate::postgres;

pub struct Server {
    pub db: RwLock<Option<Pool>>,
    // When this instance last took a full backup.
    pub last_backup: Mutex<Option<Instant>>,
}

impl Server {
    // Archives all changes to GCS, then closes the pool and stops PostgreSQL.
    pub async fn stop(&self) -> Result<(), String> {
        let mut db = self.db.write().await;
        if db.is_none() {
            return Ok(());
        }
        postgres::sync().await?;
        db.take().unwrap().close();
        postgres::stop().await?;
        println!("database handed off");
        Ok(())
    }
}

impl AsRef<Server> for Server {
    fn as_ref(&self) -> &Server {
        self
    }
}

impl ErrorHandler for Server {}

#[async_trait]
impl Default for Server {
    async fn stop(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        query_params: &StopQueryParams,
    ) -> Result<StopResponse, ()> {
        Ok(stop::stop(self, &query_params.revision).await)
    }

    async fn execute_sql(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &Vec<Statement>,
    ) -> Result<ExecuteSqlResponse, ()> {
        let db = self.db.read().await;
        Ok(execute_sql::execute_sql(db.as_ref(), body.clone()).await)
    }

    async fn health(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<HealthResponse, ()> {
        let db = self.db.read().await;
        Ok(health::health(db.is_some()).await)
    }

    async fn sync_replica(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<SyncReplicaResponse, ()> {
        let db = self.db.read().await;
        Ok(sync_replica::sync_replica(self, db.is_some()).await)
    }
}
