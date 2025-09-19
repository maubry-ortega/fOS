# FerroOS Mobile 📱

Sistema operativo móvil con pipeline **Lua → Zig → WASM → Rust**

## 🚀 Descripción

FerroOS Mobile es un sistema operativo experimental para dispositivos móviles que permite desarrollar aplicaciones usando un pipeline innovador:

1. **📝 Lua**: Interfaces de usuario escritas en Lua (simple y expresivo)
2. **🔧 Zig**: SDK que procesa y embebe Lua en módulos WASM
3. **📦 WASM/WPK**: Formato portable similar a APK pero basado en WebAssembly
4. **🦀 Rust**: Microkernel que ejecuta las aplicaciones en ARM64

## 🛠️ Requisitos

- **Zig** >= 0.14.1
- **Rust** (nightly)
- **Cargo** con soporte para `aarch64-unknown-none`

## 🏗️ Build System

El proyecto utiliza un Makefile avanzado que automatiza todo el pipeline de build.

### Comandos Principales

```bash
# Build completo con demo interactiva
make all

# Build sin demo
make pipeline

# Build rápido (solo WASM)
make quick

# Ejecutar tests del sistema
make test

# Preparar release para dispositivos
make install

# Limpiar archivos generados
make clean

# Ver ayuda completa
make help
```

### Targets Específicos

```bash
# Verificar herramientas instaladas
make check-tools

# Solo sincronizar archivo Lua
make sync-lua

# Solo generar WASM
make wasm

# Solo compilar microkernel
make mk-build

# Solo crear paquete WPK
make wpk-build
```

## 📱 Desarrollo de Aplicaciones

### 1. Crear Aplicación Lua

Edita `app.lua` con tu interfaz:

```lua
print("─── MI APLICACIÓN ───")
print("🚀 Iniciando...")
print("✅ Lista para usar")
```

### 2. Build Automático

```bash
make all
```

El sistema automáticamente:
- Embebe tu script Lua en WASM
- Crea el paquete WPK
- Compila el microkernel
- Ejecuta demo interactiva

### 3. Estructura del Proyecto

```
fos/
├── app.lua                 # Tu aplicación móvil
├── Makefile               # Sistema de build
├── sdk/                   # SDK de Zig
│   ├── src/wasm_app.zig   # Procesador Lua→WASM
│   └── build.zig          # Configuración Zig
├── wpk/                   # Paquetes WPK
│   ├── manifest.toml      # Metadatos del paquete
│   └── app.wasm          # WASM generado
├── microkernel/           # OS base en Rust
│   └── src/
│       ├── main.rs        # Kernel principal
│       └── wasm_runner.rs # Runtime WASM
└── build/release/         # Archivos finales
```

## 🎯 Flujo de Trabajo

1. **Desarrollo**: Escribe UI en `app.lua`
2. **Build**: `make all` procesa todo el pipeline
3. **Test**: `make test` verifica el sistema
4. **Release**: `make install` prepara archivos finales

## 📦 Formato WPK

Los paquetes WPK contienen:
- **app.wasm**: Aplicación compilada
- **manifest.toml**: Metadatos y permisos
- **assets/**: Recursos adicionales

### Ejemplo de Manifest

```toml
name = "app"
id = "com.fos.app"
version = "0.1.0"
entry = "app.wasm"
permissions = ["log"]
min_platform = "fOS:0.1"
```

## 🔧 Configuración Avanzada

### Variables del Makefile

```make
TARGET = aarch64-unknown-none    # Arquitectura objetivo
LUA_APP = app.lua               # Aplicación principal
WASM_OUTPUT = app.wasm          # Archivo WASM generado
```

### Targets de Compatibilidad

El Makefile mantiene compatibilidad con versiones anteriores:

```bash
make mk-build    # Solo microkernel
make mk-run      # Ejecutar con QEMU
make sdk-build   # Solo SDK
make wpk-pack    # Empaquetar WPK
```

## 🧪 Testing

```bash
# Tests completos del sistema
make test

# Verificar herramientas
make check-tools

# Demo interactiva
make demo
```

## 📱 Deployment

```bash
# Preparar release
make install

# Los archivos quedan en build/release/:
# - app.wasm (aplicación)
# - fos-microkernel.bin (kernel para dispositivo)
# - manifest.toml (metadatos)
```

## 🎉 Demo

La demo interactiva muestra paso a paso:
1. Código Lua original
2. Procesamiento con SDK Zig
3. Generación WASM/WPK
4. Ejecución en microkernel Rust
5. Resultado final funcionando

```bash
make all  # Incluye demo automática
# o
make demo # Solo demo
```

## 🚀 Próximos Pasos

- [ ] Soporte para más funciones Lua
- [ ] Runtime WASM más completo
- [ ] Driver de pantalla real
- [ ] Sistema de archivos
- [ ] Networking básico

## 📄 Licencia

Proyecto experimental para investigación y desarrollo.

---

**FerroOS Mobile** - El futuro de los sistemas operativos móviles 🚀
