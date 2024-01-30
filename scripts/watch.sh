#!/bin/sh
set -e

# run processes in parallel
# https://stackoverflow.com/a/52033580
(trap 'kill 0' SIGINT;
  # start frontend
  trunk serve -w src/frontend/ &
  # # start backend, with separate target folder to avoid rebuilds from arch change
  CARGO_TARGET_DIR=target/backend cargo watch -c -x run
)
