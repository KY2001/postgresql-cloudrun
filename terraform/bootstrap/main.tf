# Environments keep their state here, one prefix per environment.
terraform {
  required_version = ">= 1.15"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 8.0"
    }
  }
}

provider "google" {
  project = "kein-432220"
  region  = "asia-northeast1"
}

resource "google_storage_bucket" "terraform_state" {
  name     = "postgresql-cloudrun-terraform-state"
  location = "asia-northeast1"

  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  versioning {
    enabled = true
  }
}
