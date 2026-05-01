# Multi-stage build:
#   1. Compile kiln as a fully-static musl binary on rust:alpine
#   2. Copy just the binary + examples onto `scratch` for a tiny image
#
# Build:   docker build -t kiln .
# Run:     docker run --rm -p 8787:8787 kiln
# Custom:  docker run --rm -p 8787:8787 kiln /examples/http_hello.kl

FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin kiln \
 && strip target/release/kiln

FROM scratch
COPY --from=builder /src/target/release/kiln /kiln
COPY examples /examples
EXPOSE 8787
USER 1000:1000
ENTRYPOINT ["/kiln"]
CMD ["/examples/landing.kl"]
