FROM rust:1.77-slim as builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/txn-manager /app/txn-manager
COPY --from=builder /usr/src/app/migrations /app/migrations

# Create a non-root user to run the app
RUN useradd -m appuser
USER appuser

# Set the environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./txn-manager"] 