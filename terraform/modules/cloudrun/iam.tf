# The service reaches Google APIs through this account's application default credentials.
resource "google_service_account" "cloudrun" {
  account_id   = local.service
  display_name = "Cloud Run service account for postgresql-cloudrun (${var.env})"
}

# pgBackRest reads and writes the backups.
resource "google_storage_bucket_iam_member" "backup" {
  bucket = google_storage_bucket.backup.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.cloudrun.email}"
}
