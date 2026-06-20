FROM rust:1.85-bookworm AS frontend-build
RUN apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk
WORKDIR /usr/src/workspace/frontend
RUN npm install && trunk build --release

FROM rust:1.85-bookworm AS backend-build
WORKDIR /usr/src/workspace
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN openssl rand -out backend/src/secret.key 32
RUN cargo build --release -p jheffmedia-site-backend
RUN cp target/release/jheffmedia-site-backend /usr/local/bin/jheffmedia-site-backend

FROM debian:bookworm-slim
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
