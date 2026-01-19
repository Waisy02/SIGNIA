variable "name" {
  type = string
  default = "signia-api"
}

variable "image" {
  type = string
  description = "Container image reference"
}

variable "replicas" {
  type = number
  default = 2
}

variable "env" {
  type = map(string)
  default = {}
}

variable "port" {
  type = number
  default = 8787
}
