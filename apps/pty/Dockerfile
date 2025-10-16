FROM rust:1.81 as build
WORKDIR /app
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates bash coreutils && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/isolated-exec /usr/local/bin/isolated-exec
COPY config /app/config
ENV PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/isolated-exec"]
