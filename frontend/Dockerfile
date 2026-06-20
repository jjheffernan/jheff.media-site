FROM rust:1.85-bookworm
ENV RUST_LOG=info
WORKDIR /usr/src/workspace
RUN cargo install --locked trunk
COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
WORKDIR /usr/src/workspace/frontend
EXPOSE 8000
CMD ["trunk", "serve", "--port", "8000", "--address", "0.0.0.0"]
