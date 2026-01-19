# SIGNIA Terraform

This directory provides Terraform scaffolding for deploying:
- API service
- Console (web)
- Interface service
- Storage (SQLite/PVC or managed DB)
- Networking (ingress / DNS)

The modules are intentionally minimal and provider-agnostic.
You should adapt them to your target platform (AWS/GCP/Azure/Kubernetes).

## Structure
- `modules/`
  - `api/` - API deployment knobs (image, env, ports)
  - `storage/` - storage configuration (local PVC or managed db endpoint wiring)
  - `networking/` - ingress/DNS knobs
- `environments/`
  - `dev/` - sample dev environment composition
  - `prod/` - sample prod environment composition

## Quickstart
```bash
cd infra/terraform/environments/dev
terraform init
terraform plan
```

Note: These modules do not create cloud resources by default. They are templates.
