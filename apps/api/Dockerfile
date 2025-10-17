FROM rust:1.83

WORKDIR /app
COPY . .
RUN cargo build --release

EXPOSE 8081
CMD ["./target/release/agent-api"]
