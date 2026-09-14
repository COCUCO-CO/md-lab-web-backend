# Architecture

## Request path

```
Browser
  │  https://cerebrito.space/api/*
  ▼
CloudFront (TLS termination, security headers)
  │  behavior: /api/* → api-alb origin (https-only, TTLs = 0 → not cached)
  ▼
Application Load Balancer  md-lab-web-api  (443, ACM cert for cerebrito.space)
  │  target group: port 8080, health check GET /api/health
  ▼
ECS Fargate task  md-lab-web-backend  (ARM64, awsvpc, private subnets)
  └── Axum HTTP server on :8080
```

The SPA lives on a separate origin (S3 + CloudFront function rewrite) under the
same distribution; only `/api/*` reaches the ALB. Because both are same-origin,
the frontend needs no CORS configuration and the API origin request policy
forwards no cookies or extra headers (`md-lab-web-api-origin`).

## Network topology

Dedicated VPC `10.73.0.0/16` (isolated from every other project):

| Subnet            | CIDR          | Role                          |
| ----------------- | ------------- | ----------------------------- |
| `md-lab-web-public-a/b`   | 10.73.1/2.0/24   | ALB subnets (public)          |
| `md-lab-web-private-a/b`  | 10.73.101/102.0/24 | Fargate tasks (private)     |

Tasks have no public IPs; egress (ECR image pulls, CloudWatch logs) goes
through a single NAT gateway in `public-a`. The task security group accepts
port 8080 **from the ALB security group only**.

## Why ECS Fargate ARM64

- **Cheapest professional always-on option**: no instance to patch or size;
  you pay per vCPU-second/GB-second of the task. At 0.25 vCPU / 512 MB the
  compute cost is a few dollars per month (the ALB and NAT gateway dominate).
- **Graviton (ARM64)**: same Fargate price as x86 for more performance; the
  static musl binary makes the image ~4 MB, so pull times are negligible.
- **Isolation**: `awsvpc` network mode gives each task its own ENI and
  security-group membership.

## Image build strategy

The Dockerfile cross-compiles without QEMU emulation:

1. `FROM --platform=$BUILDPLATFORM rust:1.98-slim-bookworm` — the builder runs
   natively on the x86_64 CI runner (fast).
2. `cargo build --release --target aarch64-unknown-linux-musl` with
   `-C linker=rust-lld` — Rust's bundled LLD links the static musl binary
   directly, so no system cross-toolchain is needed.
3. `FROM gcr.io/distroless/static-debian12:nonroot` — final ARM64 stage with
   just the stripped binary (~4 MB). No shell, no package manager, runs as
   UID 65532.

Release flags (`lto`, `strip`, `codegen-units = 1`) keep the binary ~1.6 MB on
disk. `MDLAB_VERSION` and `GIT_SHA` are baked in at build time and served by
`GET /api/version`.

## Statelessness

The service holds no state: no database, no local disk writes, no session
storage. Any task can serve any request, which is what makes Fargate + ALB
health checks + rolling deploys safe. The MD pipeline data is static and lives
in code (`src/lib.rs`), versioned with the binary.

## Ownership split (Terraform vs CI)

| Owned by Terraform (`md-lab-web-infra`)            | Owned by CI (`deploy.yml`)                          |
| ------------------------------------------------- | --------------------------------------------------- |
| VPC, subnets, NAT, security groups                | Task definition revisions (image = git SHA)         |
| ECR repository + lifecycle                        | ECS service create (first deploy) / update (after)  |
| ALB, listener, target group                       | Image build + push                                  |
| ECS cluster, task roles, log group                | Release tag + GitHub Release                        |
| CloudFront origin/behavior for `/api/*`           |                                                       |

CI owns the mutable runtime state (task definitions, service) so
`terraform plan` never fights it; Terraform owns everything static. The SSM
parameter `/md-lab-web/prod/backend-deploy` hands CI the subnet/SG/target-group
coordinates it needs.
