FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/Nutomic/ibis"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

ARG UNAME=ibis
ARG UID=1000
ARG GID=1000

RUN groupadd -g ${GID} -o ${UNAME} && \
    useradd -m -u ${UID} -g ${GID} -o -s /bin/bash ${UNAME} && \
    apt update && apt install --yes libpq-dev
USER $UNAME

COPY target/release/ibis /usr/local/bin/ibis
RUN ls -la /usr/local/bin/ibis

ENTRYPOINT ["ibis"]
EXPOSE 8081
STOPSIGNAL SIGTERM