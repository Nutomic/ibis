#!/bin/sh
set -e

# You have to add the following lines to /etc/hosts to make this work:
#
# 127.0.0.1       ibis-alpha
# 127.0.0.1       ibis-beta
#
# Then run this script and open http://ibis-alpha:8070/, http://ibis-beta:8080/ in your browser.

DB_FOLDER="$(pwd)/target/federation_db"
mkdir -p "$DB_FOLDER/"
ALPHA_DB_PATH="$DB_FOLDER/alpha"
BETA_DB_PATH="$DB_FOLDER/beta"

# TODO: shouldnt wipe/recreate data if folder already exists
./tests/scripts/start_dev_db.sh $ALPHA_DB_PATH
./tests/scripts/start_dev_db.sh $BETA_DB_PATH

ALPHA_DB_URL="postgresql://ibis:password@/ibis?host=$ALPHA_DB_PATH"
BETA_DB_URL="postgresql://ibis:password@/ibis?host=$BETA_DB_PATH"
echo $ALPHA_DB_URL

# get rid of processes leftover from previous runs
killall ibis || true

CARGO_TARGET_DIR=target/frontend trunk build

# launch a couple of local instances to test federation
# sometimes ctrl+c doesnt work properly, so you have to kill trunk, cargo-watch and ibis manually
(trap 'kill 0' SIGINT;
  sh -c "IBIS__BIND=127.0.0.1:8070 IBIS__FEDERATION__DOMAIN=ibis-alpha:8070 IBIS__DATABASE_URL=$ALPHA_DB_URL cargo run" &
  sh -c "IBIS__BIND=127.0.0.1:8080 IBIS__FEDERATION__DOMAIN=ibis-beta:8080 IBIS__DATABASE_URL=$BETA_DB_URL cargo run" &
)