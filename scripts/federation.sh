#!/bin/sh
set -e

# launch a couple of local instances to test federation
# TODO: somehow instances use wrong port resulting in cors errors
(trap 'kill 0' SIGINT;
  sh -c 'TRUNK_SERVE_PORT=8070 IBIS_BACKEND_PORT=8071 IBIS_DATABASE_URL="postgres://ibis:password@localhost:5432/ibis" ./scripts/watch.sh' &
  sh -c 'TRUNK_SERVE_PORT=8080 IBIS_BACKEND_PORT=8081 ./scripts/watch.sh' &
  sh -c 'TRUNK_SERVE_PORT=8090 IBIS_BACKEND_PORT=8091 ./scripts/watch.sh'
)