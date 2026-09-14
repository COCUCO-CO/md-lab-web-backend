# Contributing

## Workflow

- Branch per task (`NNN-slug`), conventional commits, squash-merge to `main`.
- **Pushes to `main` deploy immediately** and cut a release — only merge what
  you would ship.
- Pull requests run `checks.yml` (fmt, clippy, tests, audit) and must be green.

## TDD rules

1. **The test comes first.** New behavior starts as a failing test in
   `tests/api.rs` (public contract) or as a unit test next to the code.
2. Run it and watch it fail for the right reason (not a compile error you
   misread).
3. Implement the minimum code that passes.
4. Refactor with the suite green; `cargo clippy --all-targets -- -D warnings`
   and `cargo fmt` must be clean.

The integration tests in `tests/api.rs` are the contract: status codes, JSON
shapes, and ordering are asserted explicitly. If a test feels awkward to write,
the API shape is probably wrong — fix the shape before implementing.

## Conventions

- Handlers live in `src/lib.rs` behind `router()` so tests never bind a port.
  Keep it that way; `main.rs` only does tracing + bind + serve.
- No new runtime dependencies without a reason written in the PR description.
  The binary is statically linked and the image is ~4 MB — both are features.
- JSON responses: camelCase-free, snake_case fields (Rust convention), no
  enums-as-strings unless an API consumer needs them.
- Version/SHA come from build args (`MDLAB_VERSION`, `GIT_SHA`) via
  `option_env!` — never hardcode a version in code.

## Local setup

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo test && cargo run        # API on http://localhost:8080/api/health
```

The toolchain (1.98.1 + clippy + rustfmt) is pinned in `rust-toolchain.toml`;
CI uses the same version, so local green means CI green.

## Definition of done

- [ ] New/changed behavior has tests written first and passing
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all clean
- [ ] `docs/api.md` updated if the public API changed
- [ ] No secrets, no committed build artifacts (`/target` is gitignored)
