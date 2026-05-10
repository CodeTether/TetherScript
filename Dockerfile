# Multi-stage build:
#   1. Compile tetherscript as a fully-static musl binary on rust:alpine
#   2. Copy just the binary + examples onto `scratch` for a tiny image
#
# Build:   docker build -t tetherscript .
# Run:     docker run --rm -p 8787:8787 tetherscript
# Custom:  docker run --rm -p 8787:8787 tetherscript /examples/http_hello.tether

FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin tetherscript \
 && strip target/release/tetherscript

FROM scratch
COPY --from=builder /src/target/release/tetherscript /tetherscript
COPY examples /examples
EXPOSE 8787
USER 1000:1000
ENTRYPOINT ["/tetherscript"]
CMD ["/examples/landing.tether"]
