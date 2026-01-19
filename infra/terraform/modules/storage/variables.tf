variable "mode" {
  type = string
  description = "storage mode: sqlite|managed"
  default = "sqlite"
}

variable "sqlite_path" {
  type = string
  default = "/data/signia.sqlite"
}

variable "object_store_mode" {
  type = string
  description = "fs|s3"
  default = "fs"
}

variable "bucket" {
  type = string
  default = ""
}
