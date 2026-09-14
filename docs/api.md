# API reference

Base URL: `https://cerebrito.space/api`

All endpoints are read-only (`GET`), return JSON, and require no
authentication today. Responses carry the standard security headers (CSP,
HSTS, X-Frame-Options) applied by CloudFront.

## `GET /api/health`

Liveness probe. Used by the ALB target group health check and by ECS; also a
cheap way to verify a deploy from outside.

```sh
curl https://cerebrito.space/api/health
# {"status":"ok"}
```

- **200** — service is up.

## `GET /api/version`

Build identity of the running binary. The version is the release tag CI baked
in at build time; `git_sha` is the commit the image was built from.

```sh
curl https://cerebrito.space/api/version
# {"name":"md-lab-web-backend","version":"v0.1.0","git_sha":"9f2c1ab..."}
```

| Field       | Type   | Notes                                    |
| ----------- | ------ | ---------------------------------------- |
| `name`      | string | Always `md-lab-web-backend`              |
| `version`   | string | Release tag (e.g. `v0.1.0`); `dev` locally |
| `git_sha`   | string | Full commit SHA; `unknown` locally       |

## `GET /api/pipeline`

The five-phase MD pipeline, in order. This is the canonical data the SPA
renders (the frontend currently ships the same list statically).

```sh
curl https://cerebrito.space/api/pipeline
# {
#   "phases": [
#     {"number":"01","title":"System build","description":"Prepare molecular systems — ..."},
#     {"number":"02","title":"Equilibration","description":"Relax each system step by step ..."},
#     {"number":"03","title":"Production MD","description":"Run long GROMACS simulations ..."},
#     {"number":"04","title":"Analysis","description":"Extract trajectories, observables ..."},
#     {"number":"05","title":"Reporting","description":"Ship artifacts, decision records ..."}
#   ]
# }
```

| Field           | Type   | Notes                          |
| --------------- | ------ | ------------------------------ |
| `phases`        | array  | Exactly five elements, in order |
| `phases[].number`     | string | Zero-padded phase number (`01`–`05`) |
| `phases[].title`      | string | Short phase name               |
| `phases[].description`| string | One-sentence description       |

## Errors

Unknown paths (including anything outside `/api/*`, which is not routed to
this service at all) return **404**. There are no other error shapes today;
when the API starts taking input, errors will follow
`{"error": "<message>"}` with the appropriate 4xx status.

## Adding an endpoint (TDD)

1. Add a failing test in `tests/api.rs` describing the contract.
2. Implement the handler in `src/lib.rs` and register the route in `router()`.
3. `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt`.
4. Push to `main` — CI deploys and releases automatically.
