
FROM rust:1.83 AS build
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/agent-api /usr/local/bin/agent-api
ENV RUST_LOG=debug
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/agent-api"]
