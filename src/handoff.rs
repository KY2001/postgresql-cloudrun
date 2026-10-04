use std::time::Duration;

use reqwest::StatusCode;

const STOP_TIMEOUT: Duration = Duration::from_secs(10);

// Asks the revision serving traffic to hand over the database.
pub async fn stop_serving_revision() {
    let (Ok(endpoint), Ok(revision)) = (std::env::var("ENDPOINT"), std::env::var("K_REVISION"))
    else {
        return; // Not running on Cloud Run (e.g. `make run`).
    };
    let result = reqwest::Client::new()
        .post(format!("https://{endpoint}/stop?revision={revision}"))
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .timeout(STOP_TIMEOUT)
        .send()
        .await;
    match result {
        Ok(response) if response.status().is_success() => println!("stopped the serving revision"),
        Ok(response) if response.status() == StatusCode::NOT_FOUND => println!("stopped nothing"),
        Err(e) if e.is_timeout() => println!("stopped nothing: {e}"),
        result => panic!("stop the serving revision: {result:?}"),
    }
}
