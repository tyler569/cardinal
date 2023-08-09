#!/usr/bin/env bash

set -euo pipefail

stdio_usage="-serial stdio"

while getopts ":im" opt; do
  case ${opt} in
    i)
      stdio_usage="-d int -serial unix:./serial,nowait,server"
      ;;
    m)
      stdio_usage="-monitor stdio -serial unix:./serial,nowait,server"
      ;;
    \?)
      echo "Usage: run.bash [-im]"
      exit 1
      ;;
  esac
done

#  -display none \

# shellcheck disable=SC2086
qemu-system-x86_64 \
  -cdrom ./cardinal3.iso \
  -vga std \
  -smp 2 \
  -m 128M \
  -netdev user,id=nic \
  -device rtl8139,netdev=nic \
  -object filter-dump,id=f1,netdev=nic,file=nic.pcap \
  $stdio_usage \
  -no-reboot
