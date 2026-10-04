# With request-based billing PostgreSQL gets no CPU between requests, so this job calls /sync to
# archive pending WAL to GCS and take an hourly full backup (and keep the instance warm).
# One job in the service's region calls once per run; an uptime check would call from at least
# three locations.
resource "google_cloud_scheduler_job" "sync" {
  name             = "${local.service}-sync"
  region           = var.gcp_region
  schedule         = var.sync_schedule
  time_zone        = "Etc/UTC"
  attempt_deadline = "60s"

  http_target {
    http_method = "POST"
    uri         = "${google_cloud_run_v2_service.server.uri}/sync"
  }
}
