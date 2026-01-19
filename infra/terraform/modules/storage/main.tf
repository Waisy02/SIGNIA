terraform {
  required_version = ">= 1.5.0"
}

resource "null_resource" "storage" {
  triggers = {
    mode = var.mode
    sqlite_path = var.sqlite_path
    object_store_mode = var.object_store_mode
    bucket = var.bucket
  }
}

output "sqlite_path" { value = var.sqlite_path }
output "object_store_mode" { value = var.object_store_mode }
