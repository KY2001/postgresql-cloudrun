use std::time::Instant;

use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::HeaderValue,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

// Request bodies can hold user data and large source code, so only log the start of them.
const MAX_LOGGED_BODY_BYTES: usize = 4096;

pub async fn log_request(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().to_string();

    let (parts, body) = request.into_parts();
    let (request_body, mut response) =
        match Bytes::from_request(Request::from_parts(parts.clone(), body), &()).await {
            Ok(body) => {
                let request_body = truncate(&String::from_utf8_lossy(&body));
                let request = Request::from_parts(parts, Body::from(body));
                (Some(request_body), next.run(request).await)
            }
            Err(error) => (None, error.into_response()),
        };
    let latency = start.elapsed();
    let status = response.status().as_u16();
    let severity = match status {
        500..=599 => "ERROR",
        400..=499 => "WARNING",
        300..=399 => "NOTICE",
        _ => "INFO",
    };

    println!(
        "{}",
        json!({
            "method": method.as_str(),
            "uri": uri,
            "request_body": request_body,
            "status": status,
            "latency": format!("{latency:?}"),
            "severity": severity,
        })
    );

    // Lets clients tell server time from network time.
    let timing = format!("app;dur={:.3}", latency.as_secs_f64() * 1000.0);
    response
        .headers_mut()
        .insert("server-timing", HeaderValue::from_str(&timing).unwrap());
    response
}

fn truncate(body: &str) -> String {
    if body.len() <= MAX_LOGGED_BODY_BYTES {
        return body.to_string();
    }
    let end = body.floor_char_boundary(MAX_LOGGED_BODY_BYTES);
    format!("{}... ({} bytes truncated)", &body[..end], body.len() - end)
}
