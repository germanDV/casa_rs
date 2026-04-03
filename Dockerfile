FROM rust:1.92 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data
WORKDIR /app
COPY --from=builder /app/target/release/casa .
COPY --from=builder /app/templates ./templates
EXPOSE 3000
CMD ["./casa"]
