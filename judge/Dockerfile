FROM rust:slim AS builder

# Build
WORKDIR /usr/src/judge
COPY . .
RUN cargo build --release 

FROM debian:stable-slim

# Add runtime dependencies
RUN apt update && apt upgrade -y
RUN apt install -y gcc g++ python3
RUN rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/judge/target/release/judge /usr/local/bin/judge

# Configure
WORKDIR /judge
EXPOSE 8000

# Run
CMD ["judge"]
