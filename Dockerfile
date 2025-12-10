# Multi-stage Docker build for MCP Services
# Stage 1: Build the Rust application
FROM rust:latest as builder

# Install required components
RUN rustup component add rustfmt clippy

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy only necessary files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build only the main binary
RUN cargo build --release --bin mcp-server-template-app

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/mcp-template-app /app/

# Set the user
USER appuser

# Expose the port the app runs on

# Expose the port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/mcp -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}' \
        || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Command to run the application
CMD ["/app/mcp-server-template-app"]