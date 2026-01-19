terraform {
  required_version = ">= 1.5.0"
}

# Provider-agnostic template.
# You should replace this with your target provider resources (kubernetes_deployment, ecs_service, cloudrun, etc).

resource "null_resource" "api" {
  triggers = {
    name     = var.name
    image    = var.image
    replicas = tostring(var.replicas)
    port     = tostring(var.port)
    env      = jsonencode(var.env)
  }
}

output "api_name" { value = var.name }
output "api_port" { value = var.port }
