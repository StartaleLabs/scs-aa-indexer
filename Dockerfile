# # syntax=docker/dockerfile:1

# FROM rust:1.86.0-alpine AS builder

# # Install dependencies
# RUN apk update
# RUN apk add ca-certificates
# RUN apk add --no-cache pkgconf musl-dev openssl-dev cmake clang bash libgcc g++ git gcc libc-dev librdkafka-dev librdkafka curl libc-dev build-base perl

# # Set working dir
# WORKDIR /app

# COPY . .

# ARG SERVICE
# RUN cargo build --release --bin ${SERVICE}

# # ===

# FROM alpine:latest
# LABEL maintainer="developer@startale.com"
# WORKDIR /app

# ARG SERVICE
# COPY --from=builder /app/target/release/${SERVICE} /app/bin/service

# RUN chmod +x /app/bin/service

# RUN addgroup -g 10000 docker && \
    # adduser -u 10000 -G docker -D -s /bin/sh -h /app docker && \
    # chown docker:docker -R /app

# USER docker

# CMD ["/app/bin/service"]

# syntax=docker/dockerfile:1

FROM rust:1.86.0-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
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
