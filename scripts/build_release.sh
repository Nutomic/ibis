#!/bin/sh
set -e

cargo leptos build
gzip target/release/ibis -c > ibis.gz
