resource "google_artifact_registry_repository" "server" {
  repository_id = local.service
  location      = var.gcp_region
  format        = "DOCKER"
  description   = "Docker repository for postgresql-cloudrun (${var.env})"

  cleanup_policies {
    id     = "delete-old-versions"
    action = "DELETE"

    condition {
      tag_state = "ANY"
    }
  }

  cleanup_policies {
    id     = "keep-last-20"
    action = "KEEP"

    most_recent_versions {
      keep_count = 20
    }
  }
}
