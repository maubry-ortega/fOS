#!/usr/bin/env bash
set -euo pipefail

qemu-system-aarch64 \
  -M raspi3b \
  -m 1024 \
  -kernel fos-microkernel.bin \
  -serial stdio \
  -display sdl


