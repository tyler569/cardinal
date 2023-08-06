#!/usr/bin/env bash

set -euo pipefail
set -x

iso_name="cardinal3.iso"
kernel_file="cardinal3"

cd kernel
mkdir -p build

# Prevent macos from creating phantom metadata files in tar archives
# https://unix.stackexchange.com/a/9865
export COPYFILE_DISABLE=1

cargo build
ld.lld -o "build/$kernel_file" \
    target/x86_64-unknown-none/debug/libcardinal3_kernel.a \
    -T link.ld -z max-page-size=0x1000

cd build

rm -rf isodir
mkdir -p isodir/boot/limine

[ -e limine ] || git clone https://github.com/limine-bootloader/limine.git \
    --branch=v4.x-branch-binary --depth=1
make -C limine

cp ./"$kernel_file" isodir/boot/"$kernel_file"
cp ../limine.cfg isodir/boot/limine/
cp ./limine/limine.sys ./limine/limine-cd.bin ./limine/limine-cd-efi.bin \
    isodir/boot/limine/

xorriso -as mkisofs -b boot/limine/limine-cd.bin \
  -no-emul-boot -boot-load-size 4 --boot-info-table \
  --efi-boot boot/limine/limine-cd-efi.bin -efi-boot-part \
  --efi-boot-image --protective-msdos-label \
  isodir -o "$iso_name"

./limine/limine-deploy "$iso_name"

cp "$iso_name" ../..
