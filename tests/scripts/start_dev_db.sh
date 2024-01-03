#!/bin/bash
set -e

export PGHOST=$1
export PGDATA="$1/dev_pgdata"

# If cluster exists, stop the backend
if [ -d $PGDATA ]
then
  # Prevent `stop` from failing if backend already stopped
  #pg_ctl restart > /dev/null
  pg_ctl stop
fi

# Remove any leftover data from revious run
rm -rf $1

# Create cluster
initdb --username=postgres --auth=trust --no-instructions

touch "$PGHOST/.s.PGSQL.5432"
echo "$PGHOST/.s.PGSQL.5432"

# Start backend that only listens to socket in current directory
pg_ctl start --options="-c listen_addresses= -c unix_socket_directories=$PGHOST"

# Setup database
psql -c "CREATE USER lemmy WITH PASSWORD 'password' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE lemmy WITH OWNER lemmy;" -U postgres
