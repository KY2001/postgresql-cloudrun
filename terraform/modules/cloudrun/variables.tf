variable "env" {
  description = "Environment name (dev or prod)"
  type        = string

  validation {
    condition     = contains(["dev", "prod"], var.env)
    error_message = "env must be dev or prod."
  }
}

variable "gcp_project_id" {
  description = "GCP project ID"
  type        = string
}

variable "gcp_region" {
  description = "GCP region"
  type        = string
}

# --- Instance Configuration ---

variable "instance_cpu" {
  description = "CPU limit for each instance (e.g. 1, 2)"
  type        = string
}

variable "instance_memory" {
  description = "Memory limit for each instance (e.g. 512Mi, 1Gi)"
  type        = string
}

variable "execution_environment" {
  description = "Execution environment (EXECUTION_ENVIRONMENT_GEN1 or EXECUTION_ENVIRONMENT_GEN2)"
  type        = string
}

variable "startup_cpu_boost" {
  description = "Whether to allocate extra CPU while an instance starts"
  type        = bool
}

variable "ephemeral_disk_size" {
  description = "Size of the ephemeral disk mounted at /data, which holds the database (e.g. 1Gi)"
  type        = string
}

# --- Scaling ---

variable "scaling_min_instance_count" {
  description = "Minimum number of instances"
  type        = number
}

variable "scaling_max_instance_count" {
  description = "Maximum number of instances; must be 1, since only one instance may own the database"
  type        = number

  validation {
    condition     = var.scaling_max_instance_count == 1
    error_message = "scaling_max_instance_count must be 1."
  }
}

variable "max_instance_request_concurrency" {
  description = "Maximum concurrent requests per instance"
  type        = number
}

# --- Request Timeout ---

variable "request_timeout" {
  description = "Request timeout in seconds (maximum 3600)"
  type        = string
}

# --- Startup Probe ---

variable "startup_probe_period_seconds" {
  description = "Startup probe period in seconds"
  type        = number
}

variable "startup_probe_timeout_seconds" {
  description = "Startup probe timeout in seconds"
  type        = number
}

variable "startup_probe_failure_threshold" {
  description = "Number of consecutive failed startup probes before the revision is abandoned"
  type        = number
}

# --- Replication ---

variable "gcs_bucket_name" {
  description = "Cloud Storage bucket pgBackRest backs the database up to"
  type        = string
}

# --- Uptime Check ---

variable "uptime_check_period" {
  description = "How often the uptime check calls /sync (60s, 300s, 600s or 900s)"
  type        = string
}
