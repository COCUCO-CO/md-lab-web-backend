# md-lab-web-backend

Lightweight, fast Rust/Axum API for the MD Lab web platform. Deployed on
**AWS ECS Fargate (ARM64/Graviton)** in account `346425562059`, region
`us-east-1`, and served at `https://cerebrito.space/api/*` through the same
CloudFront distribution as the SPA (no CORS, single TLS edge).

## Stack

- Rust 1.98 + Axum 0.8 (async HTTP)
- Sentry (`sentry` 0.49, ureq+rustls transport) — errors, panics, and request
  spans via `sentry-tracing`. No-op locally without `SENTRY_DSN`.
- Static musl binary cross-compiled for ARM64 — no runtime dependencies
- distroless container image (~6 MB)
- ECS Fargate: 0.25 vCPU / 512 MB, desired count 1

## Environment variables

| Variable            | Required | Effect                                                     |
| ------------------- | -------- | ---------------------------------------------------------- |
| `PORT`              | no       | Listen port (default `8080`)                               |
| `RUST_LOG`          | no       | Log filter (default `info`)                                |
| `SENTRY_DSN`        | no       | Sentry DSN; empty/unset → Sentry fully disabled (local dev) |
| `SENTRY_ENVIRONMENT`| no       | `environment` tag on events (default `production`)         |

`MDLAB_VERSION` / `GIT_SHA` are baked in at build time and used as the Sentry
release (`<version>+<sha>`), the same identity `GET /api/version` reports.

## Commands

```sh
cargo test          # integration + unit tests (TDD: write the test first)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run           # local server on http://localhost:8080
PORT=9000 cargo run # custom port
```

## API

| Method | Path            | Purpose                                        |
| ------ | --------------- | ---------------------------------------------- |
| GET    | `/api/health`   | Liveness probe (ALB target group + ECS)        |
| GET    | `/api/version`  | Name, release version, git SHA of the binary   |
| GET    | `/api/pipeline` | The five-phase MD pipeline, in order           |

Full reference with examples: [`docs/api.md`](docs/api.md).

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — request path, image build strategy, why Fargate ARM
- [`docs/api.md`](docs/api.md) — endpoint reference
- [`docs/deployment.md`](docs/deployment.md) — CI/CD flow, releases and tagging, rollback
- [`docs/operations.md`](docs/operations.md) — logs, scaling, troubleshooting
- [`docs/contributing.md`](docs/contributing.md) — TDD workflow and conventions

## Repository layout

```
src/lib.rs          # router + handlers (unit-testable without a port)
src/main.rs         # entrypoint: tracing setup, bind, serve
tests/api.rs        # integration tests defining the public API contract
task-definition.json# ECS task definition template (CI fills image + roles)
Dockerfile          # x86 builder -> ARM64 static musl binary -> distroless
```

## Related repositories

- [`COCUCO-CO/md-lab-web-frontend`](https://github.com/COCUCO-CO/md-lab-web-frontend) — the SPA served at `/`
- [`COCUCO-CO/md-lab-web-infra`](https://github.com/COCUCO-CO/md-lab-web-infra) — Terraform: VPC, ECR, ALB, ECS cluster, IAM, CloudFront `/api/*` behavior
