terraform {
  required_version = ">= 1.15"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 8.0"
    }
  }
}

data "google_project" "project" {
  project_id = var.gcp_project_id
}

locals {
  service = "postgresql-cloudrun-${var.env}"
}
