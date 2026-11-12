# syntax=docker/dockerfile:1.7
# Coolify Dockerfile-only: install/coolify/README.md (local git mirror + env.example)
# CSS: standalone Tailwind binary. Trunk: GitHub release binary (not cargo install).
# BuildKit cache mounts keep cargo registry/git/target warm across rebuilds.
ARG RUST_IMAGE=docker.io/library/rust:1.89-bookworm

FROM ${RUST_IMAGE} AS frontend-build
ARG TARGETARCH
ARG TRUNK_VERSION=0.21.14
ARG TAILWIND_VERSION=4.3.5
# Pin Trunk: 0.22+ needs rustc 1.96; keep in sync with image rustc.
RUN case "${TARGETARCH}" in \
      amd64) trunk_arch=x86_64; tw_arch=x64 ;; \
      arm64) trunk_arch=aarch64; tw_arch=arm64 ;; \
      *) echo "unsupported TARGETARCH=${TARGETARCH}" >&2; exit 1 ;; \
    esac \
 && curl -fsSL -o /tmp/trunk.tgz \
      "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-${trunk_arch}-unknown-linux-gnu.tar.gz" \
 && tar -xzf /tmp/trunk.tgz -C /usr/local/bin trunk \
 && chmod +x /usr/local/bin/trunk \
 && curl -fsSL -o /usr/local/bin/tailwindcss \
      "https://github.com/tailwindlabs/tailwindcss/releases/download/v${TAILWIND_VERSION}/tailwindcss-linux-${tw_arch}" \
 && chmod +x /usr/local/bin/tailwindcss \
 && rustup target add wasm32-unknown-unknown \
 && rm -f /tmp/trunk.tgz

WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p frontend/src backend/src \
 && printf '\n' > frontend/src/lib.rs \
 && echo 'fn main() {}' > backend/src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo fetch

COPY frontend ./frontend
WORKDIR /usr/src/workspace/frontend
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/usr/src/workspace/target,sharing=locked \
    trunk build --release

FROM ${RUST_IMAGE} AS backend-build
WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p frontend/src backend/src \
 && printf '\n' > frontend/src/lib.rs \
 && echo 'fn main() {}' > backend/src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo fetch

COPY backend ./backend
# Binary must be copied out in the same RUN as the cache mount (target/ is not in the image layer).
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/usr/src/workspace/target,sharing=locked \
    cargo build --release -p jheffmedia-site-backend \
 && mkdir -p /out \
 && cp target/release/jheffmedia-site-backend /out/jheffmedia-site-backend

FROM docker.io/library/debian:bookworm-slim
ARG YEW_FULLSTACK_BUILD_ID=
ENV YEW_FULLSTACK_STATIC="/usr/local/share/yew-fullstack/www"
ENV YEW_FULLSTACK_HOST="0.0.0.0"
ENV YEW_FULLSTACK_PORT="8080"
ENV RUST_LOG="info"
ENV YEW_FULLSTACK_BUILD_ID=${YEW_FULLSTACK_BUILD_ID}
# Fixed UID matches host SERVICE_UID (jheffmedia) and compose `user:` — no root at runtime.
# curl: Compose healthcheck hits GET /healthz. JWT secret is never COPY'd — mount/env at runtime.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --system --home-dir /nonexistent --shell /usr/sbin/nologin app \
    && mkdir -p /data/content-uploads \
    && chown -R app:app /data/content-uploads
COPY --from=backend-build /out/jheffmedia-site-backend /usr/local/bin/jheffmedia-site-backend
WORKDIR ${YEW_FULLSTACK_STATIC}
COPY --from=frontend-build --chown=app:app /usr/src/workspace/frontend/dist/ ${YEW_FULLSTACK_STATIC}/
USER app
EXPOSE 8080
CMD ["/usr/local/bin/jheffmedia-site-backend"]
