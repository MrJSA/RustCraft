# Same Debian release as the runtime stage — a newer builder links against a glibc the runtime doesn't have
FROM rust:1-bookworm AS builder

WORKDIR /usr/src/backend

# Pre-fetch and cache dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy real source code and migrations
COPY src ./src
COPY migrations ./migrations
COPY assets ./assets
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# pg_dump for the automatic backups — its major version must match the db image (postgres:16)
RUN install -d /usr/share/postgresql-common/pgdg \
    && curl -fsSL -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc https://www.postgresql.org/media/keys/ACCC4CF8.asc \
    && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
    && apt-get update && apt-get install -y --no-install-recommends postgresql-client-16 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/backend/target/release/backend /app/backend
COPY --from=builder /usr/src/backend/migrations /app/migrations

ENV PORT=8000
ENV RUST_LOG=info

EXPOSE 8000

CMD ["/app/backend"]
