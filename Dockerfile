# PolyTorus Containerlab Docker Image
FROM rust:1.83-bullseye as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create polytorus user
RUN useradd -ms /bin/bash polytorus

# Create necessary directories
RUN mkdir -p /data/polytorus && chown -R polytorus:polytorus /data/polytorus

# Copy binary from builder
COPY --from=builder /app/target/release/polytorus /usr/local/bin/polytorus

# Switch to polytorus user
USER polytorus

# Set working directory
WORKDIR /home/polytorus

# Create data directory
RUN mkdir -p data

# Expose ports
EXPOSE 7000 8080

# Default command
CMD ["polytorus", "--help"]
