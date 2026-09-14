# syntax=docker/dockerfile:1
# MD Lab web backend — ARM64 (Graviton) image.
#
# The builder stage runs on the BUILD platform (native x86_64 on CI runners)
# and cross-compiles a static musl binary for aarch64, so no QEMU emulation
# is needed and builds stay fast. The final stage is distroless: no shell,
# no package manager, ~7 MB image.

FROM --platform=$BUILDPLATFORM rust:1.98-slim-bookworm AS builder
RUN rustup target add aarch64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Baked into the binary and served by GET /api/version.
ARG MDLAB_VERSION=dev
ARG GIT_SHA=unknown
ENV MDLAB_VERSION=${MDLAB_VERSION} \
    GIT_SHA=${GIT_SHA}

# rust-lld (shipped with the toolchain) links the static musl binary directly,
# so no system cross-toolchain is required.
RUN RUSTFLAGS="-C linker=rust-lld" \
    cargo build --release --target aarch64-unknown-linux-musl

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/md-lab-web-backend /md-lab-web-backend
EXPOSE 8080
ENTRYPOINT ["/md-lab-web-backend"]
