#!/usr/bin/env bash

set -euo pipefail

file_name="build/cardinal3"
rustfilt="rustfilt"

while getopts "f:un" opt; do
  case $opt in
    f)
      file_name="$OPTARG"
      ;;
    u)
      file_name="build/userland"
      ;;
    n)
      rustfilt="cat"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      ;;
  esac
done

llvm-objdump -dS $file_name 2>/dev/null | $rustfilt | less
