#!/bin/sh
set -e

IBIS__BIND="${IBIS_BIND:-"127.0.0.1:8081"}"

# run processes in parallel
# https://stackoverflow.com/a/52033580
(trap 'kill 0' INT;
  # start frontend
  CARGO_TARGET_DIR=target/frontend trunk serve -w src/frontend/ --proxy-backend http://$IBIS__BIND &
  # start backend, with separate target folder to avoid rebuilds from arch change
  bacon -j run
)
