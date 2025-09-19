#!/bin/bash
# FerroOS Mobile - Test de gráficos simple

echo "🎨 Probando FerroOS Mobile con gráficos..."

# Compilar sistema
echo "📦 Compilando sistema..."
make install

# Probar con QEMU simple pero funcional
echo "🚀 Iniciando QEMU con display..."

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 512M \
    -kernel build/release/fos-microkernel.bin \
    -device virtio-gpu-pci \
    -display sdl \
    -serial stdio \
    -nic none \
    -monitor none

echo "✅ Demo completado"
