#!/bin/bash
set -e

export PGHOST=$1
export PGDATA="$1/dev_pgdata"

pg_ctl stop --mode immediate --silent

rm -r "$PGHOST"