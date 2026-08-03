# Docker image for tetherscript
#
# Build and push:
#   docker buildx build --platform linux/amd64,linux/arm64 -t codetether/tetherscript:latest -t codetether/tetherscript:0.1.0-alpha.26 --push .
#
# Or build locally:
#   docker build -t tetherscript .
#   docker run --rm tetherscript --version
#   docker run --rm tetherscript run /examples/hello.tether
#   docker run --rm -it tetherscript repl
#   docker run --rm -i tetherscript lsp
#
# With optional TLS support:
#   docker build --build-arg FEATURES=openssl-tls -t tetherscript:tls .

# --- Stage 1: build --------------------------------------------------------

FROM rust:1.97-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ARG FEATURES=""
RUN cargo build --release --bin tetherscript \
        ${FEATURES:+--features ${FEATURES}} \
 && strip target/release/tetherscript

# --- Stage 2: runtime ------------------------------------------------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/tetherscript /usr/local/bin/tetherscript
COPY examples /examples
COPY README.md LICENSE-MIT /usr/local/share/tetherscript/

WORKDIR /work
ENTRYPOINT ["tetherscript"]
CMD ["--version"]
