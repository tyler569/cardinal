#!/usr/bin/env bash

set -euo pipefail

llvm-objdump -d kernel/build/cardinal3 | rustfilt | less
