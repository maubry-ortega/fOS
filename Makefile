# FerroOS (fOS) - Makefile principal
# Pipeline: Lua → Zig → WASM → Rust
# Sistema Operativo Móvil

RUSTC ?= cargo
TARGET ?= aarch64-unknown-none
ZIG ?= zig
OBJCOPY ?= $(shell command -v rust-objcopy 2>/dev/null || command -v llvm-objcopy 2>/dev/null || echo rust-objcopy)

# Variables del proyecto
MK_BIN := fos-microkernel.bin
MK_ELF := microkernel/target/$(TARGET)/release/fos_microkernel
LUA_APP := app.lua
WASM_OUTPUT := app.wasm
WPK_DIR := wpk
SDK_DIR := sdk
DEMO_EXEC := demo_pipeline

# Colores para output
GREEN := \033[0;32m
BLUE := \033[0;34m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

.PHONY: all setup mk-build mk-run kernel-run sdk-build wasm wpk-build wpk-pack wpk-run clean \
        banner check-tools sync-lua demo test install help distclean build quick pipeline \
        emulator deploy flash ota package distribute mobile-test

# Target principal - Pipeline completo
all: banner check-tools pipeline demo
	@printf "$(GREEN)🚀 FerroOS Mobile - Build completado exitosamente!$(NC)\n"

# Pipeline completo sin demo
pipeline: sync-lua sdk-build wpk-build mk-build

# Build básico (mantener compatibilidad)
build: mk-build

setup:
	rustup target add aarch64-unknown-none
	rustup component add llvm-tools || rustup component add llvm-tools-preview || true
	cargo install cargo-binutils --locked || true

mk-build: wpk-build
	@printf "$(BLUE)🦀 COMPILANDO MICROKERNEL$(NC)\n"
	@echo "─────────────────────────"
	cd microkernel && RUSTC_BOOTSTRAP=1 $(RUSTC) build -Zbuild-std=core,compiler_builtins,alloc --release --target $(TARGET)
	$(OBJCOPY) --strip-all -O binary $(MK_ELF) $(MK_BIN)
	@printf "$(GREEN)✅ Microkernel listo: $(MK_BIN) ($$(wc -c < $(MK_BIN) 2>/dev/null || echo '?') bytes)$(NC)\n"
	@echo

mk-run: mk-build
	bash scripts/run-mk.sh

kernel-run:
	bash scripts/run.sh

sdk-build: sync-lua
	@printf "$(BLUE)🔧 COMPILANDO SDK DE ZIG$(NC)\n"
	@echo "──────────────────────"
	cd sdk && zig build -Doptimize=ReleaseFast wasm
	@if [ -f "$(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT)" ]; then \
		printf "$(GREEN)✅ SDK compilado exitosamente: $$(wc -c < $(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT)) bytes$(NC)\n"; \
	else \
		printf "$(RED)❌ Error compilando SDK$(NC)\n"; \
		exit 1; \
	fi
	@echo

wasm: sync-lua
	@printf "$(BLUE)🔧 GENERANDO WASM$(NC)\n"
	@echo "────────────────────"
	cd sdk && zig build -Doptimize=ReleaseFast wasm
	@if [ -f "$(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT)" ]; then \
		printf "$(GREEN)✅ WASM generado: $$(wc -c < $(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT)) bytes$(NC)\n"; \
	else \
		printf "$(RED)❌ Error generando WASM$(NC)\n"; \
		exit 1; \
	fi
	@echo

wpk-build: wasm
	@printf "$(BLUE)📦 CREANDO PAQUETE WPK$(NC)\n"
	@echo "───────────────────────"
	@cp $(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT) $(WPK_DIR)/$(WASM_OUTPUT)
	@cp $(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT) ./$(WASM_OUTPUT)
	@printf "$(GREEN)✅ WPK creado exitosamente$(NC)\n"
	@if [ -f "$(WPK_DIR)/manifest.toml" ]; then \
		echo "📄 Información del WPK:"; \
		echo "   Nombre: $$(grep '^name' $(WPK_DIR)/manifest.toml | cut -d'"' -f2)"; \
		echo "   ID: $$(grep '^id' $(WPK_DIR)/manifest.toml | cut -d'"' -f2)"; \
		echo "   Versión: $$(grep '^version' $(WPK_DIR)/manifest.toml | cut -d'"' -f2)"; \
	fi
	@if [ -f "scripts/wpk-pack.sh" ]; then bash scripts/wpk-pack.sh app; fi
	@echo

wpk-pack: wpk-build

wpk-run:
	@echo "[stub] wpk-run: el microkernel aún no carga WASM; pending runtime"

clean:
	rm -f $(MK_BIN)
	cd microkernel && $(RUSTC) clean
	cd sdk && zig build clean || true
	rm -f $(WASM_OUTPUT) $(DEMO_EXEC)

# ===== NUEVOS TARGETS DEL PIPELINE =====

# Banner del proyecto
banner:
	@printf "$(BLUE)"
	@echo "=== FERROOS MOBILE - BUILD SYSTEM ==="
	@echo "📱 Sistema Operativo Móvil"
	@echo "Pipeline: Lua → Zig → WASM → Rust"
	@echo "=====================================$(NC)"
	@echo

# Verificar herramientas necesarias
check-tools:
	@printf "$(BLUE)🔍 VERIFICANDO HERRAMIENTAS$(NC)\n"
	@echo "─────────────────────────"
	@command -v $(ZIG) >/dev/null 2>&1 || { printf "$(RED)❌ Zig no está instalado$(NC)\n"; exit 1; }
	@printf "$(GREEN)✅ Zig: $$($(ZIG) version)$(NC)\n"
	@command -v $(RUSTC) >/dev/null 2>&1 || { printf "$(RED)❌ Rust/Cargo no está instalado$(NC)\n"; exit 1; }
	@printf "$(GREEN)✅ Rust: $$(rustc --version | cut -d' ' -f2)$(NC)\n"
	@echo

# Extraer WASM de WPK para app de usuario
wpk-extract:
	@printf "$(BLUE)📦 EXTRAYENDO APP DE WPK$(NC)\n"
	@echo "──────────────────────────"
	@if [ ! -f "apps/user.wpk" ]; then \
		printf "$(YELLOW)⚠️  user.wpk no encontrado, creando desde ejemplo...$(NC)\n"; \
		cd examples && ../tools/fos -cP hello.lua && mv hello.wpk ../apps/user.wpk && cd ..; \
	fi
	@unzip -q -o apps/user.wpk -d build/wpk_temp
	@cp build/wpk_temp/app.wasm app.wasm
	@rm -rf build/wpk_temp
	@printf "$(GREEN)✅ app.wasm extraído de WPK ($$(wc -c < app.wasm) bytes)$(NC)\n"
	@echo

# Compilar y ejecutar demo
demo:
	@printf "$(BLUE)🎉 DEMO INTERACTIVA$(NC)\n"
	@echo "─────────────────"
	@if [ -f "demo.rs" ]; then \
		rustc demo.rs -o $(DEMO_EXEC); \
		printf "$(GREEN)✅ Demo compilada$(NC)\n"; \
		printf "$(YELLOW)🚀 Ejecutando pipeline completo...$(NC)\n"; \
		echo; \
		./$(DEMO_EXEC); \
	else \
		printf "$(YELLOW)⚠️  demo.rs no encontrado, saltando demo$(NC)\n"; \
	fi
	@echo

# Tests del sistema
test: pipeline
	@printf "$(BLUE)🧪 EJECUTANDO TESTS$(NC)\n"
	@echo "───────────────────"
	@echo "🔍 Verificando formato WASM..."
	@if command -v hexdump >/dev/null 2>&1; then \
		MAGIC=$$(hexdump -C $(WASM_OUTPUT) | head -1 | cut -d' ' -f2-5); \
		if [ "$$MAGIC" = "00 61 73 6d" ]; then \
			printf "$(GREEN)✅ Magic number WASM válido$(NC)\n"; \
		else \
			printf "$(RED)❌ Magic number WASM inválido$(NC)\n"; \
		fi; \
	fi
	@echo "🔍 Verificando script Lua embebido..."
	@if strings $(WASM_OUTPUT) | grep -q "print("; then \
		printf "$(GREEN)✅ Script Lua encontrado en WASM$(NC)\n"; \
	else \
		printf "$(RED)❌ Script Lua no encontrado$(NC)\n"; \
	fi
	@echo "🔍 Verificando microkernel..."
	@if [ -f "$(MK_ELF)" ]; then \
		printf "$(GREEN)✅ Microkernel compilado correctamente$(NC)\n"; \
	else \
		printf "$(RED)❌ Microkernel no encontrado$(NC)\n"; \
	fi
	@echo

# Target de instalación (simulada)
install: pipeline
	@printf "$(BLUE)📱 INSTALANDO FERROOS MOBILE$(NC)\n"
	@echo "────────────────────────────"
	@mkdir -p build/release
	@cp $(SDK_DIR)/zig-out/bin/$(WASM_OUTPUT) build/release/
	@cp $(WPK_DIR)/manifest.toml build/release/
	@cp $(MK_ELF) build/release/
	@cp $(MK_BIN) build/release/ 2>/dev/null || true
	@printf "$(GREEN)✅ Archivos copiados a build/release/$(NC)\n"
	@printf "$(GREEN)📱 FerroOS Mobile listo para dispositivos$(NC)\n"
	@echo

# Limpieza profunda
distclean: clean
	@printf "$(YELLOW)🧹 LIMPIEZA PROFUNDA$(NC)\n"
	@echo "──────────────────"
	@rm -f $(WPK_DIR)/$(WASM_OUTPUT)
	@rm -f $(SDK_DIR)/src/assets/$(LUA_APP)
	@rm -f $(WPK_DIR)/assets/$(LUA_APP)
	@rm -rf build/
	@printf "$(GREEN)✅ Limpieza profunda completada$(NC)\n"
	@echo

# Quick build - solo lo esencial
quick: check-tools sync-lua wasm
	@printf "$(GREEN)⚡ Quick build completado$(NC)\n"

# Ayuda
help:
	@printf "$(BLUE)FerroOS Mobile - Sistema de Build$(NC)\n"
	@echo "=================================="
	@echo
	@printf "$(YELLOW)Targets principales:$(NC)\n"
	@echo "  all         - Build completo con demo"
	@echo "  build       - Build básico (solo microkernel)"
	@echo "  pipeline    - Pipeline completo sin demo"
	@echo "  quick       - Build rápido (solo WASM)"
	@echo "  test        - Ejecutar tests del sistema"
	@echo "  clean       - Limpiar archivos generados"
	@echo "  distclean   - Limpieza profunda"
	@echo "  install     - Preparar release"
	@echo "  demo        - Solo ejecutar demo"
	@echo
	@printf "$(YELLOW)Deployment móvil:$(NC)\n"
	@echo "  emulator    - Emular dispositivo móvil con QEMU"
	@echo "  flash       - Instalar en dispositivo real"
	@echo "  distribute  - Crear paquete de distribución"
	@echo "  ota         - Crear paquete OTA"
	@echo "  deploy      - Deploy completo (dist + OTA)"
	@echo "  mobile-test - Tests para dispositivos móviles"
	@echo
	@printf "$(YELLOW)Targets originales (compatibilidad):$(NC)\n"
	@echo "  mk-build    - Solo compilar microkernel"
	@echo "  mk-run      - Ejecutar microkernel"
	@echo "  sdk-build   - Solo compilar SDK"
	@echo "  wasm        - Solo generar WASM"
	@echo "  wpk-build   - Solo crear WPK"
	@echo
	@printf "$(YELLOW)Targets de utilidad:$(NC)\n"
	@echo "  check-tools - Verificar herramientas"
	@echo "  sync-lua    - Sincronizar archivo Lua"
	@echo "  setup       - Configuración inicial"
	@echo
	@printf "$(YELLOW)Ejemplo de uso:$(NC)\n"
	@echo "  make all    - Build completo"
	@echo "  make clean  - Limpiar proyecto"
	@echo "  make test   - Verificar sistema"

# ===== TARGETS DE DEPLOYMENT MÓVIL =====

# Emular dispositivo móvil con QEMU
emulator: install
	@printf "$(BLUE)📱 EMULADOR MÓVIL$(NC)\n"
	@echo "─────────────────────"
	@./scripts/mobile-emulator.sh start

# Instalar en dispositivo real
flash: install
	@printf "$(BLUE)🔥 INSTALACIÓN EN DISPOSITIVO$(NC)\n"
	@echo "─────────────────────────────────"
	@./scripts/device-installer.sh flash

# Preparar paquete de distribución
distribute: install
	@printf "$(BLUE)📦 PAQUETE DE DISTRIBUCIÓN$(NC)\n"
	@echo "──────────────────────────────────"
	@./scripts/device-installer.sh package

# Crear paquete OTA
ota: install
	@printf "$(BLUE)📡 PAQUETE OTA$(NC)\n"
	@echo "───────────────────────"
	@./scripts/ota-updater.sh package

# Deploy completo (emulador + distribución + OTA)
deploy: install
	@printf "$(BLUE)🚀 DEPLOY COMPLETO$(NC)\n"
	@echo "──────────────────────────"
	@echo "📦 Creando paquete de distribución..."
	@./scripts/device-installer.sh package
	@echo "📡 Creando paquete OTA..."
	@./scripts/ota-updater.sh package
	@echo "🎉 Deploy completado!"
	@echo
	@echo "Archivos generados:"
	@echo "  • build/dist/ - Paquete de distribución"
	@echo "  • build/ota/  - Paquete OTA"
	@echo "  • build/firmware/ - Firmware para flasheo"

# Test en dispositivo móvil
mobile-test: install
	@printf "$(BLUE)🧪 TEST MÓVIL$(NC)\n"
	@echo "──────────────────────"
	@echo "🔍 Verificando emulador..."
	@./scripts/mobile-emulator.sh info
	@echo "📱 Detectando dispositivos..."
	@./scripts/device-installer.sh detect || echo "Sin dispositivos conectados"
	@echo "📡 Verificando actualizaciones..."
	@./scripts/ota-updater.sh check
	@echo "✅ Tests móviles completados"

# Probar gráficos con QEMU mejorado
graphics: install
	@printf "$(BLUE)🎨 MODO GRÁFICOS$(NC)\n"
	@echo "────────────────────────"
	@./scripts/qemu-graphics.sh sdl


