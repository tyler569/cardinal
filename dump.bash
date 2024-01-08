#!/usr/bin/env bash

set -euo pipefail

file_name="build/cardinal3"

while getopts "f:u" opt; do
  case $opt in
    f)
      file_name="$OPTARG"
      ;;
    u)
      file_name="build/userland"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      ;;
  esac
done

llvm-objdump -d $file_name | rustfilt | less
