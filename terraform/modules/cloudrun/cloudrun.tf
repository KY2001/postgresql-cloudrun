resource "google_cloud_run_v2_service" "server" {
  name                = local.service
  location            = var.gcp_region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false
  # Ephemeral disk is in Preview.
  launch_stage = "BETA"

  template {
    service_account                  = google_service_account.cloudrun.email
    execution_environment            = var.execution_environment
    max_instance_request_concurrency = var.max_instance_request_concurrency
    timeout                          = "${var.request_timeout}s"

    scaling {
      min_instance_count = var.scaling_min_instance_count
      max_instance_count = var.scaling_max_instance_count
    }

    volumes {
      name = "data"

      empty_dir {
        medium     = "DISK"
        size_limit = var.ephemeral_disk_size
      }
    }

    containers {
      # `make gcloud-deploy` replaces this bootstrap image with the application image once
      # Artifact Registry exists.
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      ports {
        container_port = 8080
      }

      resources {
        limits = {
          cpu    = var.instance_cpu
          memory = var.instance_memory
        }
        # Request-based billing.
        cpu_idle          = true
        startup_cpu_boost = var.startup_cpu_boost
      }

      volume_mounts {
        name       = "data"
        mount_path = "/data"
      }

      # The server listens only after it has taken the database over from the old revision.
      startup_probe {
        tcp_socket {
          port = 8080
        }

        period_seconds    = var.startup_probe_period_seconds
        timeout_seconds   = var.startup_probe_timeout_seconds
        failure_threshold = var.startup_probe_failure_threshold
      }

      env {
        name  = "PGBACKREST_REPO1_GCS_BUCKET"
        value = google_storage_bucket.backup.name
      }

      # A new instance calls https://$ENDPOINT/stop on startup. The service can't reference its
      # own URL, so this builds the deterministic one.
      env {
        name  = "ENDPOINT"
        value = "${local.service}-${data.google_project.project.number}.${var.gcp_region}.run.app"
      }
    }
  }

  lifecycle {
    ignore_changes = [
      client,
      client_version,
      template[0].containers[0].image,
      template[0].labels,
    ]
  }
}

resource "google_cloud_run_v2_service_iam_member" "public" {
  name     = google_cloud_run_v2_service.server.name
  location = google_cloud_run_v2_service.server.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}
