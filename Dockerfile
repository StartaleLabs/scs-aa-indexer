# syntax=docker/dockerfile:1

FROM rust:1.86.0-alpine AS builder

# Install dependencies
RUN apk add --no-cache pkgconfig clang lld musl-dev git openssl-dev cmake clang

# Set working dir
WORKDIR /app

COPY . .

ARG SERVICE
RUN cargo build --release --bin ${SERVICE}

# ===

FROM alpine:latest
LABEL maintainer="developer@startale.com"
WORKDIR /app

ARG SERVICE
COPY --from=builder /app/target/release/${SERVICE} /app/bin/service

RUN chmod +x /app/bin/service

RUN addgroup -g 10000 docker && \
    adduser -u 10000 -G docker -D -s /bin/sh -h /app docker && \
    chown docker:docker -R /app

USER docker

CMD ["/app/bin/service"]
