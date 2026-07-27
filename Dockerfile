FROM rust:bookworm AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY settings ./settings

RUN cargo build \
    --locked \
    --release \
    --bin personal_journal


FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        curl \
        libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system journal \
    && useradd \
        --system \
        --gid journal \
        --home-dir /app \
        --shell /usr/sbin/nologin \
        journal

WORKDIR /app

COPY --from=builder \
    /build/target/release/personal_journal \
    /app/personal_journal

COPY --from=builder /build/static /app/static
COPY --from=builder /build/settings /app/settings

RUN chown -R journal:journal /app

USER journal

EXPOSE 8123

HEALTHCHECK \
    --interval=30s \
    --timeout=5s \
    --start-period=15s \
    --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:8123/healthz || exit 1

CMD ["./personal_journal"]