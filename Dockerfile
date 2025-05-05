# syntax=docker/dockerfile:1

FROM rust:1.81.0-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    build-essential \
    librdkafka-dev \
    libssl-dev \
    zlib1g-dev \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ARG SERVICE
RUN cargo build --release --bin ${SERVICE}

# === Runtime ===

FROM debian:bookworm
LABEL maintainer="developer@startale.com"
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    librdkafka1 \
    zlib1g \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG SERVICE
COPY --from=builder /app/target/release/${SERVICE} /app/bin/service

RUN chmod +x /app/bin/service && \
    groupadd -g 10000 docker && \
    useradd -u 10000 -g docker -d /app -s /bin/sh docker && \
    chown -R docker:docker /app

USER docker

CMD ["/app/bin/service"]