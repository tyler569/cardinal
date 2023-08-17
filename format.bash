#!/usr/bin/env bash

set -euo pipefail

cargo -Zunstable-options -C kernel fmt
cargo -Zunstable-options -C userland fmt
cargo -Zunstable-options -C interface fmt
cargo -Zunstable-options -C allocator fmt
