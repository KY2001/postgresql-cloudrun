use std::collections::HashMap;

use axum::{body::Body, extract::*, response::Response, routing::*};
use axum_extra::extract::{CookieJar, Host, Query as QueryExtra};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use tracing::error;
use validator::{Validate, ValidationErrors};

use crate::{header, types::*};

#[allow(unused_imports)]
use crate::{apis, models};


/// Setup API Server.
pub fn new<I, A, E>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::default::Default<E> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    
{
    // build our application with a route
    Router::new()
        .route("/health",
            get(health::<I, A, E>)
        )
        .route("/sql",
            post(execute_sql::<I, A, E>)
        )
        .route("/stop",
            post(stop::<I, A, E>)
        )
        .route("/sync",
            post(sync_replica::<I, A, E>)
        )
        .with_state(api_impl)
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct ExecuteSqlBodyValidator<'a> {
          #[validate(
                  length(min = 1),
              )]
          body: &'a Vec<models::Statement>,
    }


#[tracing::instrument(skip_all)]
fn execute_sql_validation(
        body: Vec<models::Statement>,
) -> std::result::Result<(
        Vec<models::Statement>,
), ValidationErrors>
{
              let b = ExecuteSqlBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// ExecuteSql - POST /sql
#[tracing::instrument(skip_all)]
async fn execute_sql<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<Vec<models::Statement>>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::default::Default<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {




      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    execute_sql_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().execute_sql(
      
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::default::ExecuteSqlResponse::Status200_StatementsExecuted
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::default::ExecuteSqlResponse::Status400_SQLError
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::default::ExecuteSqlResponse::Status503_NoDatabaseConnectionAvailable
                                                    (body)
                                                => {
                                                  let mut response = response.status(503);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn health_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// Health - GET /health
#[tracing::instrument(skip_all)]
async fn health<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::default::Default<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {




      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    health_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().health(
      
      &method,
      &host,
      &cookies,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::default::HealthResponse::Status204_TheDatabaseIsResponsiveAndWALArchivingWorks
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                                apis::default::HealthResponse::Status503_TheDatabaseOrWALArchivingIsUnavailable
                                                => {
                                                  let mut response = response.status(503);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn stop_validation(
  query_params: models::StopQueryParams,
) -> std::result::Result<(
  models::StopQueryParams,
), ValidationErrors>
{
  query_params.validate()?;

Ok((
  query_params,
))
}
/// Stop - POST /stop
#[tracing::instrument(skip_all)]
async fn stop<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  QueryExtra(query_params): QueryExtra<models::StopQueryParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::default::Default<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {




      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    stop_validation(
        query_params,
    )
  ).await.unwrap();

  let Ok((
    query_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().stop(
      
      &method,
      &host,
      &cookies,
        &query_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::default::StopResponse::Status204_TheDatabaseWasHandedOff
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                                apis::default::StopResponse::Status409_TheCallerIsThisRevision
                                                => {
                                                  let mut response = response.status(409);
                                                  response.body(Body::empty())
                                                },
                                                apis::default::StopResponse::Status500_ArchivingFailed
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn sync_replica_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// SyncReplica - POST /sync
#[tracing::instrument(skip_all)]
async fn sync_replica<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::default::Default<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {




      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    sync_replica_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().sync_replica(
      
      &method,
      &host,
      &cookies,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::default::SyncReplicaResponse::Status204_AllChangesAreArchived
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                                apis::default::SyncReplicaResponse::Status500_ArchivingOrBackupFailed
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::default::SyncReplicaResponse::Status503_TheDatabaseWasHandedOffToANewRevision
                                                    (body)
                                                => {
                                                  let mut response = response.status(503);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[allow(dead_code)]
#[inline]
fn response_with_status_code_only(code: StatusCode) -> Result<Response, StatusCode> {
   Response::builder()
          .status(code)
          .body(Body::empty())
          .map_err(|_| code)
}
