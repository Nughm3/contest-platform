FROM rust:slim AS build

# Set up Rust project
WORKDIR /src
RUN cargo init

# Build and cache dependencies only
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --bin server --release
RUN rm -rf src

# Build binary
COPY . .
RUN touch -am src/main.rs
RUN cargo build --bin server --release

FROM debian:stable-slim AS final

# Add runtime dependencies
RUN apt update && apt install -y gcc g++ python3
RUN rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=build /src/target/release/judge /usr/local/bin/judge

# Configure
WORKDIR /judge
VOLUME ["/judge"]
EXPOSE 8128

# Run
CMD ["judge"]
