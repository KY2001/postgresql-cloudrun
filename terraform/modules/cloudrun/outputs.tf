output "artifact_registry_repository" {
  description = "Artifact Registry repository name"
  value       = google_artifact_registry_repository.server.repository_id
}

output "cloud_run_service_name" {
  description = "Cloud Run service name"
  value       = google_cloud_run_v2_service.server.name
}

output "cloud_run_service_url" {
  description = "Public Cloud Run service URL"
  value       = google_cloud_run_v2_service.server.uri
}

output "gcs_bucket_name" {
  description = "Cloud Storage bucket holding the pgBackRest backups"
  value       = google_storage_bucket.backup.name
}

output "service_account_email" {
  description = "Cloud Run runtime service account email"
  value       = google_service_account.cloudrun.email
}
