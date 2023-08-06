#!/usr/bin/env bash

llvm-objdump -d kernel/build/cardinal3 | rustfilt | less
