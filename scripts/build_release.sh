#!/bin/sh
set -e

cargo leptos build --release
gzip target/release/ibis -c > ibis.gz
