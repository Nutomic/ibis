#!/bin/bash
set -e

export PGHOST=$1
export PGDATA="$1/dev_pgdata"
echo $PGHOST

pg_ctl stop
rm -rf $PGDATA