#!/bin/bash
# FerroOS Mobile - Emulador de dispositivo móvil
# Simula un teléfono ARM64 completo con pantalla táctil

set -e

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuración del dispositivo móvil virtual
DEVICE_NAME="FerroPhone"
CPU_CORES=4
RAM_SIZE="2G"
STORAGE_SIZE="8G"
SCREEN_WIDTH=1080
SCREEN_HEIGHT=1920

# Rutas
KERNEL_IMG="build/release/fos-microkernel.bin"
BOOTLOADER="build/release/bootloader.bin"
SYSTEM_IMG="build/release/system.img"
USERDATA_IMG="build/release/userdata.img"

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════╗"
    echo "║        FerroOS Mobile Emulator       ║"
    echo "║     📱 Simulador de Dispositivo      ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}"
}

check_dependencies() {
    echo -e "${YELLOW}🔍 Verificando dependencias...${NC}"
    
    if ! command -v qemu-system-aarch64 &> /dev/null; then
        echo -e "${RED}❌ QEMU ARM64 no está instalado${NC}"
        echo "Instalando QEMU..."
        sudo pacman -S qemu-system-aarch64 || {
            echo "Por favor instala QEMU manualmente: sudo pacman -S qemu-system-aarch64"
            exit 1
        }
    fi
    
    echo -e "${GREEN}✅ QEMU ARM64 disponible${NC}"
}

create_virtual_storage() {
    echo -e "${YELLOW}💾 Creando almacenamiento virtual...${NC}"
    
    mkdir -p build/release/virtual-device
    
    # Crear imagen del sistema
    if [ ! -f "$SYSTEM_IMG" ]; then
        echo "📦 Creando imagen del sistema (${STORAGE_SIZE})..."
        qemu-img create -f qcow2 "$SYSTEM_IMG" "$STORAGE_SIZE"
    fi
    
    # Crear imagen de datos de usuario
    if [ ! -f "$USERDATA_IMG" ]; then
        echo "📁 Creando almacenamiento de usuario (2G)..."
        qemu-img create -f qcow2 "$USERDATA_IMG" 2G
    fi
    
    echo -e "${GREEN}✅ Almacenamiento virtual listo${NC}"
}

create_bootloader() {
    echo -e "${YELLOW}🚀 Creando bootloader móvil...${NC}"
    
    # Crear un bootloader simple que cargue nuestro kernel
    cat > build/release/bootloader.S << 'EOF'
// FerroOS Mobile Bootloader
// Bootloader mínimo para dispositivos ARM64

.section ".text.boot"
.global _start

_start:
    // Configurar stack pointer
    ldr x30, =0x40080000
    mov sp, x30
    
    // Mostrar splash screen
    bl show_splash
    
    // Cargar kernel principal
    bl load_kernel
    
    // Saltar al kernel
    ldr x30, =0x40200000
    br x30

show_splash:
    // Aquí iría el código para mostrar splash screen
    // Por ahora solo retornamos
    ret

load_kernel:
    // Aquí iría el código para cargar el kernel desde storage
    // Por ahora solo retornamos  
    ret

.section ".data"
splash_msg: .ascii "FerroOS Mobile v1.0\n"
EOF
    
    # Ensamblar bootloader
    if command -v aarch64-linux-gnu-as &> /dev/null; then
        aarch64-linux-gnu-as build/release/bootloader.S -o build/release/bootloader.o
        aarch64-linux-gnu-ld build/release/bootloader.o -o build/release/bootloader.elf -Ttext=0x40000000
        aarch64-linux-gnu-objcopy -O binary build/release/bootloader.elf "$BOOTLOADER"
        echo -e "${GREEN}✅ Bootloader creado${NC}"
    else
        echo -e "${YELLOW}⚠️  Usando kernel como bootloader${NC}"
        cp "$KERNEL_IMG" "$BOOTLOADER"
    fi
}

start_emulator() {
    echo -e "${YELLOW}📱 Iniciando ${DEVICE_NAME}...${NC}"
    
    # Verificar que el kernel existe
    if [ ! -f "$KERNEL_IMG" ]; then
        echo -e "${RED}❌ Kernel no encontrado. Ejecuta 'make install' primero${NC}"
        exit 1
    fi
    
    echo "🔧 Configuración del dispositivo:"
    echo "   CPU: ARM Cortex-A72 (${CPU_CORES} cores)"
    echo "   RAM: ${RAM_SIZE}"
    echo "   Pantalla: ${SCREEN_WIDTH}x${SCREEN_HEIGHT}"
    echo "   Almacenamiento: ${STORAGE_SIZE}"
    echo
    
    # Configuración específica para dispositivo móvil
    QEMU_ARGS=(
        -machine virt
        -cpu cortex-a72
        -smp ${CPU_CORES}
        -m ${RAM_SIZE}
        -kernel "$KERNEL_IMG"
        
        # Gráficos: Usar virtio-gpu-pci, el estándar para virtualización.
        # El framebuffer se mapeará a una dirección conocida por el kernel.
        -device virtio-gpu-pci
        -display sdl,window-close=off
        
        # Almacenamiento
        -drive file="$SYSTEM_IMG",if=virtio,format=qcow2
        -drive file="$USERDATA_IMG",if=virtio,format=qcow2
        
        # Conectividad
        -netdev user,id=net0,hostfwd=tcp::5555-:5555
        -device virtio-net-pci,netdev=net0
        
        # Audio (para llamadas/multimedia)
        -device intel-hda
        -device hda-duplex
        
        # USB (para conectar dispositivos)
        -device qemu-xhci,id=xhci
        -device usb-tablet,bus=xhci.0
        
        # Serial para debugging
        -serial stdio
        
        # Monitor para comandos
        -monitor telnet:127.0.0.1:4444,server,nowait
    )
    
    echo -e "${GREEN}🚀 Arrancando emulador...${NC}"
    echo "   Monitor: telnet localhost 4444"
    echo "   ADB: adb connect localhost:5555"
    echo "   Para salir: Ctrl+A, X"
    echo
    
    qemu-system-aarch64 "${QEMU_ARGS[@]}"
}

show_device_info() {
    echo -e "${BLUE}📱 Información del Dispositivo Virtual${NC}"
    echo "═══════════════════════════════════════"
    echo "Nombre: $DEVICE_NAME"
    echo "Arquitectura: ARM64 (AArch64)"
    echo "Procesador: ARM Cortex-A72 ($CPU_CORES cores)"
    echo "Memoria RAM: $RAM_SIZE"
    echo "Almacenamiento: $STORAGE_SIZE"
    echo "Pantalla: ${SCREEN_WIDTH}x${SCREEN_HEIGHT} (táctil)"
    echo "SO: FerroOS Mobile"
    echo "Kernel: $(ls -lh $KERNEL_IMG 2>/dev/null | awk '{print $5}' || echo 'No disponible')"
    echo
}

# Función principal
main() {
    case "${1:-start}" in
        start)
            print_header
            show_device_info
            check_dependencies
            create_virtual_storage
            create_bootloader
            start_emulator
            ;;
        info)
            show_device_info
            ;;
        clean)
            echo -e "${YELLOW}🧹 Limpiando dispositivo virtual...${NC}"
            rm -rf build/release/virtual-device
            rm -f build/release/*.img build/release/*.qcow2
            echo -e "${GREEN}✅ Dispositivo virtual limpiado${NC}"
            ;;
        *)
            echo "Uso: $0 [start|info|clean]"
            echo "  start - Iniciar emulador (por defecto)"
            echo "  info  - Mostrar información del dispositivo"
            echo "  clean - Limpiar archivos del dispositivo virtual"
            ;;
    esac
}

main "$@"
