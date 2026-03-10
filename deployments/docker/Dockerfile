# syntax=docker/dockerfile:1

# ── Build stage ──────────────────────────────────────────
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p core

# ── Runtime stage ────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/false katara
USER katara
WORKDIR /app

COPY --from=builder /app/target/release/core /app/katara-core

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD ["/app/katara-core", "--version"] || exit 1

CMD ["/app/katara-core"]
