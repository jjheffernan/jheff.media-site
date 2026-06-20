FROM rust:latest
ENV RUST_LOG=info
WORKDIR /usr/src/workspace
RUN cargo install cargo-watch
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
# Populate workspace member metadata so `cargo fetch` can resolve the workspace.
RUN mkdir -p backend/src frontend/src && \
    echo 'fn main() {}' > backend/src/main.rs && \
    echo '' > frontend/src/lib.rs
RUN cargo fetch
EXPOSE 8080
CMD ["cargo", "watch", "-x", "run -p jheffmedia-site-backend --features forward-frontend"]
