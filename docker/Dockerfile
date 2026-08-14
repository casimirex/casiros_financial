# Multi-stage build for the casiros-api server.
#
# Builder base is rust:1.88-slim-bookworm, not the spec's 1.82: this
# workspace's rust-toolchain.toml/rust-version were bumped to 1.88 in Phase 5
# because actix-web 4.14's ecosystem (actix-http, actix-server, actix-ws,
# bytestring, time) requires it. See crates/api/Cargo.toml's own comments for
# why actix-ws (not actix-web-actors) is used.
FROM rust:1.88-slim-bookworm AS builder

# curl + ca-certificates: utoipa-swagger-ui's build.rs downloads the actual
# Swagger UI static assets at build time via the system `curl` binary (its
# "reqwest" feature is off, matching crates/api/Cargo.toml's plain
# `utoipa-swagger-ui = { version = "9", features = ["actix-web"] }`).
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml ./
COPY .cargo ./.cargo
COPY crates ./crates

RUN cargo build --release --locked -p casiros-api

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 1000 --no-create-home --shell /usr/sbin/nologin casiros

COPY --from=builder /app/target/release/casiros-api /usr/local/bin/casiros-api
COPY config/ /etc/casiros/

USER 1000
EXPOSE 8080
ENV CASIROS_SERVER__BIND_ADDR=0.0.0.0:8080

ENTRYPOINT ["/usr/local/bin/casiros-api"]
