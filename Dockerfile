# Multi-stage build for Guild Workspace
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build all workspace members
RUN cargo build --workspace --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /app/target/release/guild-home /usr/local/bin/
COPY --from=builder /app/target/release/guild-api /usr/local/bin/
COPY --from=builder /app/target/release/guild-storage /usr/local/bin/

# Default ports
EXPOSE 8000 3000

# Default command
CMD ["guild-home"]