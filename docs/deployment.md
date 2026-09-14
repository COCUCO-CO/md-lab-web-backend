# Deployment

## Pipeline overview

Two workflows in `.github/workflows/`:

### `checks.yml` — pull requests and pushes to main

```
checkout → rust 1.98.1 (clippy, rustfmt)
         → cargo fmt --check
         → cargo clippy --all-targets -- -D warnings
         → cargo test
         → cargo audit          (RustSec advisory DB, prebuilt binary)
```

Nothing is deployed; a PR must be green before merge.

### `deploy.yml` — push to `main` (GitHub environment: `production`)

```
1. Compute release version
     no previous tag            → v0.1.0
     previous tag vX.Y.Z        → vX.Y.(Z+1)
2. Assume md-lab-web-BackendDeployRole (OIDC, repo-pinned)
3. Read deploy coordinates from SSM /md-lab-web/prod/backend-deploy
4. Build linux/arm64 image (buildx; x86 builder cross-compiles — no QEMU)
     tags: <ecr>/<sha>  and  <ecr>/<version>
5. Register ECS task definition revision (image = <sha>)
6. First deploy: ecs create-service · afterwards: ecs update-service
7. aws ecs wait services-stable
8. Smoke test through CloudFront:
     /api/health → {"status":"ok"}
     /api/version → version must equal the release tag
9. gh release create <version> --target main   (GitHub Release + git tag)
```

The deploy is serialized by a concurrency group (`backend-deploy`, no
cancel-in-progress), so overlapping pushes queue instead of racing.

## Releases and tagging

- **Every green deploy produces a tag** `vX.Y.Z` on the deployed commit and a
  GitHub Release titled `Backend vX.Y.Z` with the commit SHA, image tag, and
  public URL. The release list is the changelog.
- Patch bumps are automatic. To cut a **minor/major** release, push a manual
  tag first (e.g. `git tag v0.2.0 && git push origin v0.2.0`) — the next
  deploy then bumps from there (`v0.2.1`).
- Images are immutable and dual-tagged: by commit SHA (what ECS runs) and by
  release version (what humans reference). ECR expires untagged images after
  30 days; tagged images persist.

## Rollback

Rolling back to a previous release is one command — the old image and task
definition revision still exist:

```sh
aws ecs update-service \
  --cluster md-lab-web --service md-lab-web-backend \
  --task-definition md-lab-web-backend:<old-revision>
```

Find revisions with `aws ecs list-task-definitions --family md-lab-web-backend`.
The ALB target group health check (`/api/health`) removes unhealthy tasks
automatically, and the service has a deployment circuit breaker (see
`docs/architecture.md` for the ownership split).

## First-deploy notes

The ECS **service** is created by CI on the first deploy (Terraform creates
the cluster but not the service — see architecture). Until the first green
deploy, the ALB target group has no healthy targets and `/api/*` returns 503;
the SPA at `/` is unaffected.
