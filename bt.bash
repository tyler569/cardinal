#!/usr/bin/env bash

set -euo pipefail

file_name="build/cardinal3"

while getopts "f:u" opt; do
  case $opt in
    f)
      file_name=$OPTARG
      ;;
    u)
      file_name="build/userland"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      exit 1
      ;;
  esac
done

< last_output \
    grep '(.*) <.*>' | \
    sed 's/.*(\(.*\)).*/\1/g' | \
    grep -v -e '^0x0$' -e '^0$' | \
    xargs llvm-addr2line -fipsae "$file_name" | \
    rustfilt

