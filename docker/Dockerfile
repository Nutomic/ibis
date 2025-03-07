FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Install build deps
RUN cargo install cargo-leptos && \
    rustup target add wasm32-unknown-unknown && \
    apt update && apt install npm --yes --no-install-recommends and --no-install-suggests
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo leptos build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source="https://github.com/Nutomic/ibis"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

ARG UNAME=ibis
ARG UID=1000
ARG GID=1000

RUN groupadd -g ${GID} -o ${UNAME} && \
    useradd -m -u ${UID} -g ${GID} -o -s /bin/bash ${UNAME} && \
    apt update && apt install --yes libpq-dev
USER $UNAME

COPY --from=builder /app/target/release/ibis /usr/local/bin/ibis
ENTRYPOINT ["ibis"]
EXPOSE 3000
STOPSIGNAL SIGTERM
