FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/updater
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Docker CLI (incl. compose plugin) + git to update the checkout and rebuild the shop
FROM docker:27-cli
RUN apk add --no-cache git
COPY --from=builder /usr/src/updater/target/release/updater /usr/local/bin/updater
ENV REPO_DIR=/repo SECRETS_DIR=/secrets RUST_LOG=info
CMD ["updater"]
