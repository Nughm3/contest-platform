FROM rust:slim AS build

# Set up Rust project
WORKDIR /src
RUN cargo init

# Build and cache dependencies only
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN mkdir src/bin && cp src/main.rs src/bin/server.rs && mv src/main.rs src/bin/generate_json_schema.rs
RUN cargo build --release
RUN rm -rf src

# Build binary
COPY . .
RUN touch -am src/bin/*
RUN cargo build --bin server --release

FROM debian:stable-slim AS final

# Add runtime dependencies
RUN apt update && apt install -y gcc g++ python3 && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=build /src/target/release/server /usr/local/bin/judge-server

# Configure
WORKDIR /judge
VOLUME ["/judge"]
EXPOSE 8128

# Run
CMD ["judge-server"]
