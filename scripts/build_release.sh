#!/bin/sh
set -e

CARGO_TARGET_DIR=target/frontend trunk build --release
cargo build --release
gzip target/release/ibis -c > ibis.gz
