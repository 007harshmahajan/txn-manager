FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies and build with --release
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* && \
    cargo build --release

# Runtime stage - slim image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies and curl for healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy only the necessary files from the builder stage
COPY --from=builder /usr/src/app/target/release/txn-manager /app/txn-manager
COPY --from=builder /usr/src/app/migrations /app/migrations

# Health check to ensure the application is running
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8080/ || exit 1

# Create a non-root user to run the app
RUN useradd -m appuser
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Expose the port
EXPOSE 8080

# Run the application
CMD ["./txn-manager"] 