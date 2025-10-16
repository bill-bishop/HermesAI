FROM rust:1.81 as build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs && cargo build --release || true
COPY src ./src
COPY config ./config
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build /app/target/release/isolated-exec /usr/local/bin/isolated-exec
COPY --from=build /app/config /app/config
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/isolated-exec"]
