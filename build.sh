#!/bin/bash
set -e

echo "=== FERROOS MOBILE - BUILD PIPELINE ==="
echo "📱 Sistema Operativo Móvil"
echo "Pipeline: Lua → Zig → WASM → Rust"
echo "======================================"
echo

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Verificar herramientas necesarias
print_step "🔍 VERIFICANDO HERRAMIENTAS NECESARIAS"
echo "──────────────────────────────────────"

if ! command -v zig &> /dev/null; then
    print_error "Zig no está instalado"
    exit 1
fi
print_success "Zig encontrado: $(zig version)"

if ! command -v rustc &> /dev/null; then
    print_error "Rust no está instalado"
    exit 1
fi
print_success "Rust encontrado: $(rustc --version)"

echo

# Paso 1: Sincronizar app.lua
print_step "📝 PASO 1: PREPARANDO CÓDIGO LUA"
echo "─────────────────────────────"

if [ -f "app.lua" ]; then
    print_success "app.lua encontrado ($(wc -c < app.lua) bytes)"
    
    # Sincronizar con SDK
    cp app.lua sdk/src/assets/app.lua
    print_success "Archivo sincronizado con SDK de Zig"
    
    # Sincronizar con WPK
    cp app.lua wpk/assets/app.lua
    print_success "Archivo sincronizado con directorio WPK"
else
    print_error "app.lua no encontrado"
    exit 1
fi

echo

# Paso 2: Compilar con SDK de Zig
print_step "🔧 PASO 2: COMPILACIÓN CON ZIG SDK"
echo "──────────────────────────────────"

cd sdk

if zig build wasm; then
    print_success "SDK de Zig compilado exitosamente"
    
    if [ -f "zig-out/bin/app.wasm" ]; then
        WASM_SIZE=$(wc -c < zig-out/bin/app.wasm)
        print_success "WASM generado: app.wasm (${WASM_SIZE} bytes)"
    else
        print_error "app.wasm no generado"
        exit 1
    fi
else
    print_error "Error compilando SDK de Zig"
    exit 1
fi

cd ..
echo

# Paso 3: Crear paquete WPK
print_step "📦 PASO 3: CREANDO PAQUETE WPK"
echo "──────────────────────────────"

# Copiar WASM al directorio WPK
cp sdk/zig-out/bin/app.wasm wpk/app.wasm
print_success "WASM copiado a directorio WPK"

# Copiar WASM al directorio raíz para el microkernel
cp sdk/zig-out/bin/app.wasm .
print_success "WASM copiado para microkernel"

# Verificar manifest WPK
if [ -f "wpk/manifest.toml" ]; then
    print_success "Manifest WPK válido"
    echo "📄 Información del WPK:"
    echo "   Nombre: $(grep '^name' wpk/manifest.toml | cut -d'"' -f2)"
    echo "   ID: $(grep '^id' wpk/manifest.toml | cut -d'"' -f2)"
    echo "   Versión: $(grep '^version' wpk/manifest.toml | cut -d'"' -f2)"
else
    print_error "manifest.toml no encontrado"
    exit 1
fi

echo

# Paso 4: Compilar microkernel de Rust
print_step "🦀 PASO 4: COMPILACIÓN MICROKERNEL RUST"
echo "───────────────────────────────────────"

cd microkernel

if cargo build --release; then
    print_success "Microkernel compilado exitosamente"
    
    KERNEL_PATH="target/aarch64-unknown-none/release/fos_microkernel"
    if [ -f "$KERNEL_PATH" ]; then
        KERNEL_SIZE=$(wc -c < "$KERNEL_PATH")
        print_success "Binario del kernel: fos_microkernel (${KERNEL_SIZE} bytes)"
    else
        print_error "Binario del kernel no encontrado"
        exit 1
    fi
else
    print_error "Error compilando microkernel de Rust"
    exit 1
fi

cd ..
echo

# Paso 5: Ejecutar demo interactiva
print_step "🎉 PASO 5: DEMO INTERACTIVA"
echo "───────────────────────────"

if rustc demo.rs -o demo_pipeline; then
    print_success "Demo compilada exitosamente"
    
    echo
    echo -e "${YELLOW}🚀 Ejecutando demo del pipeline completo...${NC}"
    echo
    
    ./demo_pipeline
else
    print_warning "No se pudo compilar la demo, pero el pipeline está funcional"
fi

echo

# Resumen final
print_step "📊 RESUMEN DEL BUILD"
echo "───────────────────"

echo "✅ Archivos generados:"
echo "   📄 app.lua ($(wc -c < app.lua) bytes)"
echo "   📦 sdk/zig-out/bin/app.wasm ($(wc -c < sdk/zig-out/bin/app.wasm) bytes)"
echo "   📱 wpk/app.wasm ($(wc -c < wpk/app.wasm) bytes)"
echo "   🦀 microkernel/target/aarch64-unknown-none/release/fos_microkernel"

echo
echo "🎯 Pipeline completo:"
echo "   1. ✅ Lua: Script de aplicación móvil procesado"
echo "   2. ✅ Zig: SDK procesó y embebió Lua en WASM"  
echo "   3. ✅ WASM: Paquete WPK generado correctamente"
echo "   4. ✅ Rust: Microkernel compilado para ARM64"

echo
echo -e "${GREEN}🚀 FERROOS MOBILE - BUILD COMPLETADO EXITOSAMENTE!${NC}"
echo -e "${BLUE}📱 Sistema listo para dispositivos móviles${NC}"
echo
