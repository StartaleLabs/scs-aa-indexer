# Use official Rust image
FROM rust:1.84.0

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang curl

# Create app directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock first (for caching builds)
COPY Cargo.toml Cargo.lock ./

# Build dependencies early to cache
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the source
COPY . .

# Then run your app
CMD ["cargo", "run", "--release"]
