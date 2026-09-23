FROM rust:latest AS builder

WORKDIR /usr/src/backend

# Pre-fetch and cache dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy real source code and migrations
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/backend/target/release/backend /app/backend
COPY --from=builder /usr/src/backend/migrations /app/migrations

ENV PORT=8000
ENV RUST_LOG=info

EXPOSE 8000

CMD ["/app/backend"]
