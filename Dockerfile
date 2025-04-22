# syntax=docker/dockerfile:1

FROM rust:1.86.0-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang curl

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
COPY --from=builder /app/target/release/${SERVICE} /app/bin/${SERVICE}

RUN adduser -D -u 1000 docker
RUN chmod +x /app/bin/${SERVICE} && \
    chown -R docker:docker /app

USER docker
CMD ["/app/bin/${SERVICE}"]
