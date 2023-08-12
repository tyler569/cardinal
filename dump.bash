#!/usr/bin/env bash

set -euo pipefail

llvm-objdump -d build/cardinal3 | rustfilt | less
