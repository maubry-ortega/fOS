#!/usr/bin/env bash
set -euo pipefail

qemu-system-aarch64 \
  -M raspi4b \
  -m 2048 \
  -kernel fos-microkernel.bin \
  -serial stdio \
  -display sdl


