terraform {
  required_version = ">= 1.5.0"
}

resource "null_resource" "networking" {
  triggers = {
    domain = var.domain
    api_host = var.api_host
    console_host = var.console_host
    interface_host = var.interface_host
  }
}

output "domain" { value = var.domain }
output "hosts" {
  value = {
    api = var.api_host
    console = var.console_host
    interface = var.interface_host
  }
}
