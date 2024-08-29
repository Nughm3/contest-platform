FROM rust:slim AS builder

# Install build dependencies
RUN apt update && apt upgrade -y

# Setup Rust project
WORKDIR /usr/src
RUN cargo new judge
WORKDIR /usr/src/judge

# Build and cache dependencies only
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release
RUN rm -rf src

# Build binary
COPY src src
RUN cargo build --release

FROM debian:stable-slim AS final

# Add runtime dependencies
RUN apt update && apt upgrade -y
RUN apt install -y gcc g++ python3
RUN rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/judge/target/release/judge /usr/local/bin/judge

# Configure
WORKDIR /judge
VOLUME ["/judge"]
EXPOSE 8000

# Run
ENV RUST_LOG=trace
CMD ["judge"]
