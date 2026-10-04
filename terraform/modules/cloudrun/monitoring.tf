# With request-based billing PostgreSQL gets no CPU between requests, so this archives pending
# WAL to GCS, takes an hourly full backup (and keeps the instance warm).
resource "google_monitoring_uptime_check_config" "sync" {
  display_name = "${local.service} sync"
  period       = var.uptime_check_period
  timeout      = "60s"

  http_check {
    path           = "/sync"
    request_method = "POST"
    # The API requires a content type for POST; /sync ignores the (empty) body.
    content_type = "URL_ENCODED"
    port         = 443
    use_ssl      = true
    validate_ssl = true
  }

  monitored_resource {
    type = "uptime_url"
    labels = {
      project_id = var.gcp_project_id
      host       = trimprefix(google_cloud_run_v2_service.server.uri, "https://")
    }
  }
}
