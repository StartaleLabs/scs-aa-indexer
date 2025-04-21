FROM rust:1.84.0

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang curl

# Set working dir
WORKDIR /app

# Copy full workspace Cargo files
COPY Cargo.toml Cargo.lock ./
COPY indexer/Cargo.toml indexer/Cargo.toml
COPY api/Cargo.toml api/Cargo.toml

# Copy minimal dummy source to cache deps
RUN mkdir -p indexer/src api/src
RUN echo 'fn main() {}' > indexer/src/main.rs
RUN echo 'fn main() {}' > api/src/main.rs

# Build to cache deps
RUN cargo build --release

# Clean up dummy files
RUN rm -rf indexer/src api/src

# Copy full source
COPY . .

# Build actual binary (set by docker-compose SERVICE env)
CMD ["sh", "-c", "cargo build --release && exec target/release/$SERVICE"]
