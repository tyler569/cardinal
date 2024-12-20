#!/usr/bin/env bash

set -euo pipefail
set -x

# Prevent macos from creating phantom metadata files in tar archives
# https://unix.stackexchange.com/a/9865
export COPYFILE_DISABLE=1

iso_name="cardinal3.iso"
kernel_file="cardinal3"

mkdir -p build
build_dir="$(pwd)/build"

cargo -Zunstable-options -C userland build
cp userland/target/x86_64-unknown-none/debug/user_main "$build_dir"/userland


cargo -Zunstable-options -C kernel build
cp kernel/target/x86_64-unknown-none/debug/cardinal3-kernel "$build_dir"/cardinal3

cd "$build_dir"

rm -rf isodir
mkdir -p isodir/boot/limine

[ -e limine ] || git clone https://github.com/limine-bootloader/limine.git \
    --branch=v4.x-branch-binary --depth=1
make -C limine

cp ./"$kernel_file" isodir/boot
cp ./userland isodir/boot
cp ../kernel/limine.cfg isodir/boot/limine/
cp ./limine/limine.sys ./limine/limine-cd.bin ./limine/limine-cd-efi.bin \
    isodir/boot/limine/

xorriso -as mkisofs -b boot/limine/limine-cd.bin \
  -no-emul-boot -boot-load-size 4 --boot-info-table \
  --efi-boot boot/limine/limine-cd-efi.bin -efi-boot-part \
  --efi-boot-image --protective-msdos-label \
  isodir -o "$iso_name"

./limine/limine-deploy "$iso_name"

cp "$iso_name" ..
