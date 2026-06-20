FROM rust:1.85-bookworm
ENV RUST_LOG=info
RUN apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/workspace
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk
COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
WORKDIR /usr/src/workspace/frontend
RUN npm install && npm run css:build
EXPOSE 8000
CMD ["sh", "-c", "unset NO_COLOR; npm run css:watch & exec trunk serve --port 8000 --address 0.0.0.0"]
