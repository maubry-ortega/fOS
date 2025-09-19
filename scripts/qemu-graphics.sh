#!/bin/bash
# FerroOS Mobile - QEMU con gráficos funcionando
# Configuración específica para mostrar framebuffer

set -e

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuración
KERNEL_PATH="build/release/fos-microkernel.bin"

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════╗"
    echo "║      FerroOS Mobile - QEMU GPU       ║"
    echo "║        🎨 Gráficos Funcionando       ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}"
}

check_kernel() {
    if [ ! -f "$KERNEL_PATH" ]; then
        echo -e "${RED}❌ Kernel no encontrado: $KERNEL_PATH${NC}"
        echo "Ejecuta 'make install' primero"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Kernel encontrado: $(wc -c < "$KERNEL_PATH") bytes${NC}"
}

start_qemu_graphics() {
    echo -e "${YELLOW}🚀 Iniciando QEMU con gráficos...${NC}"
    
    # Configuración específica para ARM64 con gráficos
    echo "📱 Configuración:"
    echo "   - CPU: ARM Cortex-A72"
    echo "   - RAM: 1GB"
    echo "   - Display: 800x600 32bpp"
    echo "   - GPU: VirtIO-GPU"
    echo
    
    # Argumentos específicos para framebuffer funcionando
    QEMU_ARGS=(
        # Máquina y CPU
        -machine virt
        -cpu cortex-a72
        -smp 2
        -m 1G
        
        # Kernel
        -kernel "$KERNEL_PATH"
        
        # Display con configuración que funciona
        # Usar virtio-gpu-pci, el estándar para virtualización.
        -device virtio-gpu-pci
        -display sdl,window-close=off
        
        # Dispositivos de entrada
        -device qemu-xhci
        -device usb-tablet
        
        # Serial para debug
        -serial stdio
        
        # Sin red por simplicidad
        -nic none
        
        # Configuración de memoria
        -global virtio-mmio.force-legacy=false
        
        # Monitor deshabilitado para simplificar
        -monitor none
    )
    
    echo -e "${GREEN}🖥️  Arrancando con SDL display...${NC}"
    echo "   Para salir: Cierra la ventana o Ctrl+C en terminal"
    echo
    
    # Ejecutar QEMU
    qemu-system-aarch64 "${QEMU_ARGS[@]}"
}

start_qemu_gtk() {
    echo -e "${YELLOW}🚀 Iniciando QEMU con GTK...${NC}"
    
    QEMU_ARGS=(
        -machine virt
        -cpu cortex-a72
        -smp 2
        -m 1G
        -kernel "$KERNEL_PATH"
        
        # Display GTK
        -device virtio-gpu-pci
        -display gtk,grab-on-hover=on,zoom-to-fit=off
        
        # Dispositivos
        -device virtio-keyboard-pci
        -device virtio-mouse-pci
        
        # Debug
        -serial stdio
        -nic none
        -monitor none
    )
    
    echo -e "${GREEN}🖥️  Arrancando con GTK display...${NC}"
    qemu-system-aarch64 "${QEMU_ARGS[@]}"
}

start_qemu_vnc() {
    echo -e "${YELLOW}🚀 Iniciando QEMU con VNC...${NC}"
    
    QEMU_ARGS=(
        -machine virt
        -cpu cortex-a72
        -smp 2
        -m 1G
        -kernel "$KERNEL_PATH"
        
        # VNC display
        -device virtio-gpu-pci
        -display vnc=localhost:1
        
        # Dispositivos
        -device virtio-keyboard-pci
        -device virtio-mouse-pci
        
        # Debug
        -serial stdio
        -nic none
        -monitor none
        -daemonize
    )
    
    echo -e "${GREEN}🖥️  Arrancando con VNC...${NC}"
    echo "   VNC: localhost:5901"
    echo "   Conecta con: vncviewer localhost:5901"
    
    qemu-system-aarch64 "${QEMU_ARGS[@]}"
}

# Función principal
main() {
    case "${1:-sdl}" in
        sdl)
            print_header
            check_kernel
            start_qemu_graphics
            ;;
        gtk)
            print_header
            check_kernel
            start_qemu_gtk
            ;;
        vnc)
            print_header
            check_kernel
            start_qemu_vnc
            ;;
        *)
            echo "FerroOS Mobile - QEMU con Gráficos"
            echo
            echo "Uso: $0 [sdl|gtk|vnc]"
            echo
            echo "Opciones de display:"
            echo "  sdl  - SDL display (recomendado)"
            echo "  gtk  - GTK display"
            echo "  vnc  - VNC server"
            echo
            echo "Ejemplos:"
            echo "  $0 sdl    # Usar SDL"
            echo "  $0 vnc    # Usar VNC en puerto 5901"
            ;;
    esac
}

main "$@"
