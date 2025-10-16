# ---------- build (musl static) ----------
FROM rust:1.81-alpine AS build
RUN apk add --no-cache musl-dev build-base
WORKDIR /app

# cache deps
COPY Cargo.toml ./
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs && cargo build --release || true

# actual sources
COPY src ./src
COPY config ./config

# build static binary
RUN rustup target add x86_64-unknown-linux-musl
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---------- runtime (no shell) ----------
FROM gcr.io/distroless/static-debian12
WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/isolated-exec /isolated-exec
COPY --from=build /app/config /app/config
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["/isolated-exec"]
