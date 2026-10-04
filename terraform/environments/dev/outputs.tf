output "artifact_registry_repository" {
  description = "Artifact Registry repository name"
  value       = module.server.artifact_registry_repository
}

output "cloud_run_service_name" {
  description = "Cloud Run service name"
  value       = module.server.cloud_run_service_name
}

output "cloud_run_service_url" {
  description = "Public Cloud Run service URL"
  value       = module.server.cloud_run_service_url
}

output "gcs_bucket_name" {
  description = "Cloud Storage bucket holding the pgBackRest backups"
  value       = module.server.gcs_bucket_name
}

output "service_account_email" {
  description = "Cloud Run runtime service account email"
  value       = module.server.service_account_email
}
