# Operations

## Where to look

| What                | Where                                                                 |
| ------------------- | --------------------------------------------------------------------- |
| Container logs      | CloudWatch Logs → `/md-lab-web/backend` (retention 30 days)           |
| Task events         | ECS console → cluster `md-lab-web` → service `md-lab-web-backend`     |
| ALB health          | EC2 → Load Balancers → `md-lab-web-api` → Target groups               |
| ECR images + scans  | ECR → `md-lab-web-backend` (scan on push enabled)                     |
| Releases            | GitHub → `md-lab-web-backend` → Releases                              |
| Public behavior     | `https://cerebrito.space/api/health`, `/api/version`, `/api/pipeline` |

## Day-2 commands

```sh
# Live version (should match the latest release)
curl -s https://cerebrito.space/api/version | python3 -m json.tool

# Recent container logs
aws logs tail --region us-east-1 /md-lab-web/backend/backend --follow

# Running tasks + their task definition revision
aws ecs list-tasks --cluster md-lab-web --service md-lab-web-backend \
  && aws ecs describe-tasks --cluster md-lab-web \
     --tasks <task-arn> --query 'tasks[0].taskDefinitionArn' --output text

# Scale up (e.g. for a load test) — set back to 1 afterwards
aws ecs update-service --cluster md-lab-web --service md-lab-web-backend --desired-count 2

# Roll back to a previous revision
aws ecs list-task-definitions --family md-lab-web-backend \
  --sort DESC --query 'taskDefinitionArns[0]' --output text
aws ecs update-service --cluster md-lab-web --service md-lab-web-backend \
  --task-definition <old-revision>
```

## Scaling

The service runs at **desired count 1** (the cost baseline). The app is
stateless, so scaling is just `--desired-count N`; the ALB spreads traffic
across healthy targets. Fargate spin-up takes ~30–60 s per task. If sustained
load ever requires more than a few tasks, revisit CPU/memory sizing (currently
0.25 vCPU / 512 MB) before adding count — the binary is tiny and single-threaded
per connection pool, so memory is the likely first constraint.

## Troubleshooting

**`/api/*` returns 503 from CloudFront**
The target group has no healthy targets. Check task status in ECS:
- `PROVISIONING`/`PENDING` — normal right after a deploy (wait for stable).
- `STOPPED` with reason `Essential container in task exited` — look at the
  CloudWatch stream for that task; usually a bad image or OOM.
- Image pull errors — confirm the ECR image exists and the execution role is
  intact (`md-lab-web-BackendTaskExecutionRole`).

**Deploy stuck in `PENDING`/`ACTIVE` with old version still serving**
`aws ecs describe-services --cluster md-lab-web --services md-lab-web-backend`
shows events. A failed deployment rolls back automatically (circuit breaker).
If a task never becomes healthy, the ALB health check path is `/api/health` —
verify the new image serves it: `docker run --rm -p 8080:8080 <image>` then
`curl localhost:8080/api/health`.

**Smoke test fails but ECS is stable**
CloudFront may still be routing to a draining target; wait a minute and retry.
If it persists, confirm the `/api/*` behavior still targets the `api-alb`
origin (infra repo → `frontend.tf`) and that the ALB listener certificate is
valid (ACM cert for `cerebrito.space`).

**Version mismatch after deploy**
`/api/version` lags the release tag when a rolling update hasn't finished;
`aws ecs wait services-stable` in CI guards against this. If it happens
manually, check which task definition revision is actually running (above).

## Costs (approx., us-east-1)

| Item                              | ~$/month |
| --------------------------------- | -------- |
| Fargate 0.25 vCPU / 512 MB × 1    | 3–4      |
| ALB (hourly + LCU)                | 18–20    |
| NAT Gateway (1 AZ)                | 32       |
| ECR storage + scan, CloudWatch logs, SSM | < 5   |
| **Total**                         | **~55–65** |

The ALB and NAT dominate. If the budget ever tightens: a single-AZ VPC removes
one subnet pair (no savings on NAT), or move to an NLB (~$10/mo) — at the cost
of the standard HTTP setup documented here.
