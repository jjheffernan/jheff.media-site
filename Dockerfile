# Coolify Dockerfile-only: install/coolify/README.md (local git mirror + env.example)
# CSS uses the standalone Tailwind binary — no Node/npm in this image.
FROM docker.io/library/rust:1.89-bookworm AS frontend-build
ARG TAILWIND_VERSION=4.3.3
RUN curl -fsSL -o /usr/local/bin/tailwindcss \
      "https://github.com/tailwindlabs/tailwindcss/releases/download/v${TAILWIND_VERSION}/tailwindcss-linux-x64" \
    && chmod +x /usr/local/bin/tailwindcss
WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN rustup target add wasm32-unknown-unknown
# Pin: trunk 0.22+ needs rustc 1.96; keep in sync with image rustc.
RUN cargo install --locked trunk --version 0.21.14
WORKDIR /usr/src/workspace/frontend
RUN trunk build --release

FROM docker.io/library/rust:1.89-bookworm AS backend-build
WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN cargo build --release -p jheffmedia-site-backend
RUN cp target/release/jheffmedia-site-backend /usr/local/bin/jheffmedia-site-backend

FROM docker.io/library/debian:bookworm-slim
ENV YEW_FULLSTACK_STATIC="/usr/local/share/yew-fullstack/www"
ENV YEW_FULLSTACK_HOST="0.0.0.0"
ENV YEW_FULLSTACK_PORT="8080"
ENV RUST_LOG="info"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=backend-build /usr/local/bin/jheffmedia-site-backend /usr/local/bin/jheffmedia-site-backend
WORKDIR ${YEW_FULLSTACK_STATIC}
COPY --from=frontend-build /usr/src/workspace/frontend/dist/ ${YEW_FULLSTACK_STATIC}/
EXPOSE 8080
CMD ["/usr/local/bin/jheffmedia-site-backend"]
