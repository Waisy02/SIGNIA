terraform {
  required_version = ">= 1.5.0"
}

module "storage" {
  source = "../../modules/storage"
  mode = "sqlite"
  sqlite_path = "/data/signia-dev.sqlite"
  object_store_mode = "fs"
}

module "networking" {
  source = "../../modules/networking"
  domain = "dev.signia.example"
  api_host = "api.dev.signia.example"
  console_host = "console.dev.signia.example"
  interface_host = "interface.dev.signia.example"
}

module "api" {
  source = "../../modules/api"
  image = "signia/api:dev"
  replicas = 1
  env = {
    SIGNIA_BIND_ADDR = "0.0.0.0:8787"
    SIGNIA_SQLITE_PATH = module.storage.sqlite_path
    SIGNIA_OBJECT_STORE_MODE = module.storage.object_store_mode
  }
}
