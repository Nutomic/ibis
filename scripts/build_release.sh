#!/bin/sh
set -e

cargo leptos build --release --precompress
gzip target/release/ibis -c > ibis.gz
