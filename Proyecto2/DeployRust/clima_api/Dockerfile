FROM rust:1.86.0 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y\
    libssl3\
    ca-certificates\
    && rm -rf /var/lib/apt/lists/*
#clima-api es el nombre del proyecto en rust
COPY --from=builder /app/target/release/clima_api /usr/local/bin/clima_api   

EXPOSE 8080
CMD ["/usr/local/bin/clima_api"]