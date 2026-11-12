FROM docker.io/library/rust:1.89-bookworm
ENV RUST_LOG=info
ARG TAILWIND_VERSION=4.3.3
RUN curl -fsSL -o /usr/local/bin/tailwindcss \
      "https://github.com/tailwindlabs/tailwindcss/releases/download/v${TAILWIND_VERSION}/tailwindcss-linux-x64" \
    && chmod +x /usr/local/bin/tailwindcss
WORKDIR /usr/src/workspace
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk --version 0.21.14
COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
WORKDIR /usr/src/workspace/frontend
RUN bash build-css.sh
EXPOSE 8000
CMD ["sh", "-c", "unset NO_COLOR; bash build-css.sh --watch & exec trunk serve --port 8000 --address 0.0.0.0"]
