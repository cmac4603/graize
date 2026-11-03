FROM rust:1.86-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release -p controller

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/controller /usr/local/bin/controller
USER 10001:10001
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/controller"]
