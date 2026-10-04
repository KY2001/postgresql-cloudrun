# pgBackRest stores full backups and archived WAL here and restores them on startup.
resource "google_storage_bucket" "backup" {
  name     = var.gcs_bucket_name
  location = var.gcp_region

  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
}
