FROM rust:1.99-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release -p ui
RUN cargo install topcoat-cli --version 0.11.0 --locked
RUN topcoat asset bundle --release --package ui

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/ui /app/ui
COPY --from=builder /workspace/target/release/assets /app/assets
COPY --from=builder /workspace/ui/toasty /app/toasty
LABEL org.opencontainers.image.source="https://github.com/graize/graize"
ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:/data/data.db
USER 10001:10001
EXPOSE 3000
ENTRYPOINT ["/app/ui"]
