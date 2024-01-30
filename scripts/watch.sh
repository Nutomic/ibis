#!/bin/sh
set -e

IBIS_BACKEND_PORT="${IBIS_BACKEND_PORT:-8081}"

# run processes in parallel
# https://stackoverflow.com/a/52033580
(trap 'kill 0' SIGINT;
  # start frontend
  trunk serve -w src/frontend/ --proxy-backend http://127.0.0.1:$IBIS_BACKEND_PORT &
  # # start backend, with separate target folder to avoid rebuilds from arch change
  CARGO_TARGET_DIR=target/backend cargo watch -x run
)
