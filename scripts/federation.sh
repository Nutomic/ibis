#!/bin/sh
set -e

# You have to add the following lines to /etc/hosts to make this work:
#
# 127.0.0.1       ibis-alpha
# 127.0.0.1       ibis-beta
#
# Then run this script and open http://ibis-alpha:8070/, http://ibis-beta:8080/ in your browser.

function cleanup {
    echo "stop postgres"
    ./scripts/stop_dev_db.sh $ALPHA_DB_PATH
    ./scripts/stop_dev_db.sh $BETA_DB_PATH
}
trap cleanup EXIT

DB_FOLDER="$(pwd)/target/federation_db"
mkdir -p "$DB_FOLDER/"
ALPHA_DB_PATH="$DB_FOLDER/alpha"
BETA_DB_PATH="$DB_FOLDER/beta"

# create db folders if they dont exist
if [ ! -d $ALPHA_DB_PATH ]; then
    ./scripts/start_dev_db.sh $ALPHA_DB_PATH
else
  pg_ctl start --options="-c listen_addresses= -c unix_socket_directories=$ALPHA_DB_PATH" -D "$ALPHA_DB_PATH/dev_pgdata"
fi
if [ ! -d $BETA_DB_PATH ]; then
    ./scripts/start_dev_db.sh $BETA_DB_PATH
else
  pg_ctl start --options="-c listen_addresses= -c unix_socket_directories=$BETA_DB_PATH" -D "$BETA_DB_PATH/dev_pgdata"
fi

ALPHA_DB_URL="postgresql://ibis:password@/ibis?host=$ALPHA_DB_PATH"
BETA_DB_URL="postgresql://ibis:password@/ibis?host=$BETA_DB_PATH"

# get rid of processes leftover from previous runs
killall ibis || true

CARGO_TARGET_DIR=target/frontend trunk build

# launch a couple of local instances to test federation, then wait for processes to finish
IBIS__BIND=127.0.0.1:8070 IBIS__FEDERATION__DOMAIN=ibis-alpha:8070 IBIS__DATABASE_URL=$ALPHA_DB_URL cargo run &
PID_ALPHA=($!)
IBIS__BIND=127.0.0.1:8080 IBIS__FEDERATION__DOMAIN=ibis-beta:8080 IBIS__DATABASE_URL=$BETA_DB_URL cargo run &
PID_BETA=($!)

wait $PID_ALPHA
wait $PID_BETA
