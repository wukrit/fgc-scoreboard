FROM rust:1-bookworm AS builder
WORKDIR /app
COPY server ./server
RUN cargo build --release --manifest-path server/Cargo.toml

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/server/target/release/fgc-server /app/fgc-server
COPY web /app/web
RUN mkdir -p /app/data
ENV FGC_ASSET_ROOT=/app/web
ENV FGC_DATA_DIR=/app/data
EXPOSE 8080
CMD ["/app/fgc-server", "--no-tunnel"]
