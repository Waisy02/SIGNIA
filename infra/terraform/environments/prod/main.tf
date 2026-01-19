terraform {
  required_version = ">= 1.5.0"
}

module "storage" {
  source = "../../modules/storage"
  mode = "managed"
  sqlite_path = "/data/signia.sqlite"
  object_store_mode = "s3"
  bucket = "signia-prod-artifacts"
}

module "networking" {
  source = "../../modules/networking"
  domain = "signia.example"
  api_host = "api.signia.example"
  console_host = "console.signia.example"
  interface_host = "interface.signia.example"
}

module "api" {
  source = "../../modules/api"
  image = "signia/api:latest"
  replicas = 3
  env = {
    SIGNIA_BIND_ADDR = "0.0.0.0:8787"
    SIGNIA_OBJECT_STORE_MODE = module.storage.object_store_mode
    SIGNIA_S3_BUCKET = module.storage.bucket
  }
}
