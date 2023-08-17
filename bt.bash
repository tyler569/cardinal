#!/usr/bin/env bash

set -euo pipefail

cat last_output | \
    grep '(.*) <.*>' | \
    sed 's/.*(\(.*\)).*/\1/g' | \
    xargs llvm-addr2line -fips -e build/cardinal3 | \
    rustfilt

