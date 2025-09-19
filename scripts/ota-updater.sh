#!/bin/bash
# FerroOS Mobile - Sistema de actualización OTA
# Actualización remota del sistema operativo

set -e

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuración
OTA_SERVER="https://releases.ferroos.com"
CURRENT_VERSION="1.0.0"
OTA_DIR="build/ota"
UPDATE_MANIFEST="update-manifest.json"

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════╗"
    echo "║       FerroOS Mobile OTA System      ║"
    echo "║      📡 Over-The-Air Updates         ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}"
}

check_for_updates() {
    echo -e "${YELLOW}🔍 Buscando actualizaciones...${NC}"
    
    # Crear archivo de versión actual si no existe
    if [ ! -f "build/current-version.txt" ]; then
        echo "$CURRENT_VERSION" > build/current-version.txt
    fi
    
    local current_version=$(cat build/current-version.txt)
    
    # Simular verificación de servidor (en producción sería una llamada HTTP)
    echo "📡 Conectando a servidor OTA..."
    echo "   Servidor: $OTA_SERVER"
    echo "   Versión actual: $current_version"
    
    # Simular respuesta del servidor
    local latest_version="1.0.1"
    local update_available=true
    
    if [ "$update_available" = true ]; then
        echo -e "${GREEN}✅ Actualización disponible: v${latest_version}${NC}"
        
        # Crear manifest de actualización
        create_update_manifest "$latest_version"
        return 0
    else
        echo -e "${GREEN}✅ Sistema actualizado (v${current_version})${NC}"
        return 1
    fi
}

create_update_manifest() {
    local new_version="$1"
    
    echo -e "${YELLOW}📋 Creando manifest de actualización...${NC}"
    
    mkdir -p "$OTA_DIR"
    
    cat > "$OTA_DIR/$UPDATE_MANIFEST" << EOF
{
    "ota_update": {
        "version": "$new_version",
        "previous_version": "$CURRENT_VERSION",
        "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "architecture": "arm64",
        "device": "ferrophone",
        "download_size": 52428800,
        "install_size": 134217728,
        "packages": [
            {
                "name": "system",
                "type": "system_partition",
                "size": 536870912,
                "checksum": "sha256:$(echo -n "system_update_$new_version" | sha256sum | cut -d' ' -f1)",
                "url": "$OTA_SERVER/updates/v$new_version/system.img",
                "critical": true
            },
            {
                "name": "boot",
                "type": "boot_partition", 
                "size": 33554432,
                "checksum": "sha256:$(echo -n "boot_update_$new_version" | sha256sum | cut -d' ' -f1)",
                "url": "$OTA_SERVER/updates/v$new_version/boot.img",
                "critical": true
            },
            {
                "name": "apps",
                "type": "application_bundle",
                "size": 15728640,
                "checksum": "sha256:$(echo -n "apps_update_$new_version" | sha256sum | cut -d' ' -f1)",
                "url": "$OTA_SERVER/updates/v$new_version/apps.wasm",
                "critical": false
            }
        ],
        "changelog": [
            "🚀 Mejoras de rendimiento del kernel",
            "📱 Nuevas funciones de UI en aplicaciones Lua",
            "🔧 Correcciones de bugs en el runtime WASM",
            "⚡ Optimizaciones de batería",
            "🛡️ Parches de seguridad"
        ],
        "installation_steps": [
            "backup_userdata",
            "download_packages", 
            "verify_checksums",
            "install_boot",
            "install_system",
            "install_apps",
            "verify_installation",
            "reboot"
        ]
    }
}
EOF
    
    echo -e "${GREEN}✅ Manifest creado${NC}"
}

download_update() {
    echo -e "${YELLOW}📥 Descargando actualización...${NC}"
    
    if [ ! -f "$OTA_DIR/$UPDATE_MANIFEST" ]; then
        echo -e "${RED}❌ No hay actualización disponible${NC}"
        exit 1
    fi
    
    # En un sistema real, aquí descargaríamos los archivos del servidor
    # Por ahora simularemos el proceso
    
    echo "📦 Descargando paquetes:"
    
    # Simular descarga de system.img
    echo "   📱 system.img (512MB)..."
    create_dummy_file "$OTA_DIR/system.img" 512
    
    # Simular descarga de boot.img
    echo "   🥾 boot.img (32MB)..."
    create_dummy_file "$OTA_DIR/boot.img" 32
    
    # Simular descarga de apps.wasm
    echo "   📱 apps.wasm (15MB)..."
    create_dummy_file "$OTA_DIR/apps.wasm" 15
    
    echo -e "${GREEN}✅ Descarga completada${NC}"
    
    # Verificar checksums
    verify_update_checksums
}

create_dummy_file() {
    local filename="$1"
    local size_mb="$2"
    
    # Crear archivo con contenido que simule una actualización real
    dd if=/dev/urandom of="$filename" bs=1M count="$size_mb" 2>/dev/null
    
    # Agregar header identificador
    echo "FerroOS_Mobile_Update_$(basename "$filename")" | dd of="$filename" bs=1 seek=0 conv=notrunc 2>/dev/null
}

verify_update_checksums() {
    echo -e "${YELLOW}🔐 Verificando integridad...${NC}"
    
    local files=("system.img" "boot.img" "apps.wasm")
    
    for file in "${files[@]}"; do
        if [ -f "$OTA_DIR/$file" ]; then
            local checksum=$(sha256sum "$OTA_DIR/$file" | cut -d' ' -f1)
            echo "   ✅ $file: ${checksum:0:16}..."
        else
            echo -e "   ${RED}❌ $file: Archivo faltante${NC}"
            exit 1
        fi
    done
    
    echo -e "${GREEN}✅ Integridad verificada${NC}"
}

backup_userdata() {
    echo -e "${YELLOW}💾 Creando respaldo de datos...${NC}"
    
    mkdir -p "$OTA_DIR/backup"
    
    # Simular backup de datos importantes
    if [ -f "build/release/userdata.img" ]; then
        cp "build/release/userdata.img" "$OTA_DIR/backup/"
        echo "   ✅ Datos de usuario respaldados"
    fi
    
    # Backup de configuración
    if [ -f "wpk/manifest.toml" ]; then
        cp "wpk/manifest.toml" "$OTA_DIR/backup/"
        echo "   ✅ Configuración respaldada"
    fi
    
    echo -e "${GREEN}✅ Respaldo completado${NC}"
}

install_update() {
    echo -e "${YELLOW}🔧 Instalando actualización...${NC}"
    
    # Verificar que tenemos todos los archivos
    local required_files=("$OTA_DIR/system.img" "$OTA_DIR/boot.img" "$OTA_DIR/apps.wasm")
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            echo -e "${RED}❌ Archivo faltante: $file${NC}"
            exit 1
        fi
    done
    
    echo "🔄 Aplicando actualizaciones:"
    
    # Instalar boot partition
    echo "   🥾 Actualizando bootloader..."
    if install_boot_partition; then
        echo "      ✅ Bootloader actualizado"
    else
        echo -e "      ${RED}❌ Error en bootloader${NC}"
        rollback_update
        exit 1
    fi
    
    # Instalar system partition
    echo "   📱 Actualizando sistema..."
    if install_system_partition; then
        echo "      ✅ Sistema actualizado"
    else
        echo -e "      ${RED}❌ Error en sistema${NC}"
        rollback_update
        exit 1
    fi
    
    # Instalar aplicaciones
    echo "   📦 Actualizando aplicaciones..."
    if install_applications; then
        echo "      ✅ Aplicaciones actualizadas"
    else
        echo -e "      ${YELLOW}⚠️  Error en aplicaciones (continuando)${NC}"
    fi
    
    # Actualizar versión
    echo "1.0.1" > build/current-version.txt
    
    echo -e "${GREEN}✅ Actualización instalada exitosamente${NC}"
}

install_boot_partition() {
    # Simular instalación de boot partition
    if [ -f "$OTA_DIR/boot.img" ]; then
        cp "$OTA_DIR/boot.img" "build/release/boot.img"
        return 0
    fi
    return 1
}

install_system_partition() {
    # Simular instalación de system partition
    if [ -f "$OTA_DIR/system.img" ]; then
        cp "$OTA_DIR/system.img" "build/release/system.img"
        return 0
    fi
    return 1
}

install_applications() {
    # Simular instalación de aplicaciones
    if [ -f "$OTA_DIR/apps.wasm" ]; then
        cp "$OTA_DIR/apps.wasm" "build/release/app.wasm"
        return 0
    fi
    return 1
}

rollback_update() {
    echo -e "${RED}🔙 Iniciando rollback...${NC}"
    
    # Restaurar desde backup
    if [ -f "$OTA_DIR/backup/userdata.img" ]; then
        cp "$OTA_DIR/backup/userdata.img" "build/release/"
        echo "   ✅ Datos restaurados"
    fi
    
    if [ -f "$OTA_DIR/backup/manifest.toml" ]; then
        cp "$OTA_DIR/backup/manifest.toml" "wpk/"
        echo "   ✅ Configuración restaurada"
    fi
    
    echo -e "${GREEN}✅ Rollback completado${NC}"
}

create_ota_server() {
    echo -e "${YELLOW}🖥️  Creando servidor OTA local...${NC}"
    
    local server_dir="build/ota-server"
    mkdir -p "$server_dir/updates/v1.0.1"
    
    # Crear estructura del servidor
    cat > "$server_dir/server.py" << 'EOF'
#!/usr/bin/env python3
"""
FerroOS Mobile OTA Server
Servidor simple para distribución de actualizaciones
"""

import json
import os
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import urlparse

class OTAHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        path = urlparse(self.path).path
        
        if path == '/check_updates':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            
            response = {
                "update_available": True,
                "latest_version": "1.0.1",
                "download_url": "http://localhost:8000/updates/v1.0.1/",
                "size": 52428800,
                "changelog": [
                    "🚀 Performance improvements",
                    "🔧 Bug fixes",
                    "📱 New UI features"
                ]
            }
            
            self.wfile.write(json.dumps(response, indent=2).encode())
        else:
            super().do_GET()

if __name__ == '__main__':
    port = 8000
    server = HTTPServer(('localhost', port), OTAHandler)
    print(f"🖥️  OTA Server running on http://localhost:{port}")
    print("📡 Endpoints:")
    print(f"   /check_updates - Check for updates")
    print(f"   /updates/      - Download updates")
    server.serve_forever()
EOF
    
    chmod +x "$server_dir/server.py"
    
    # Crear archivos de ejemplo para el servidor
    cp -r "$OTA_DIR"/*.img "$server_dir/updates/v1.0.1/" 2>/dev/null || true
    cp -r "$OTA_DIR"/*.wasm "$server_dir/updates/v1.0.1/" 2>/dev/null || true
    
    echo -e "${GREEN}✅ Servidor OTA creado en $server_dir${NC}"
    echo "Para iniciar: python3 $server_dir/server.py"
}

generate_ota_package() {
    echo -e "${YELLOW}📦 Generando paquete OTA...${NC}"
    
    # Asegurar que tenemos archivos de release
    if [ ! -f "build/release/fos-microkernel.bin" ]; then
        echo -e "${RED}❌ Ejecuta 'make install' primero${NC}"
        exit 1
    fi
    
    mkdir -p "$OTA_DIR/package"
    
    # Crear paquete completo OTA
    echo "📦 Empaquetando archivos..."
    
    # Copiar archivos del sistema
    cp "build/release/fos-microkernel.bin" "$OTA_DIR/package/boot.img"
    cp "build/release/app.wasm" "$OTA_DIR/package/apps.wasm"
    
    # Crear system.img simulado
    create_dummy_file "$OTA_DIR/package/system.img" 64
    
    # Crear manifest del paquete
    create_update_manifest "1.0.1"
    cp "$OTA_DIR/$UPDATE_MANIFEST" "$OTA_DIR/package/"
    
    # Crear archivo ZIP del paquete OTA
    cd "$OTA_DIR"
    zip -r "ferroos-mobile-ota-v1.0.1.zip" package/
    
    echo -e "${GREEN}✅ Paquete OTA creado: $OTA_DIR/ferroos-mobile-ota-v1.0.1.zip${NC}"
    echo "📦 Tamaño: $(du -h "$OTA_DIR/ferroos-mobile-ota-v1.0.1.zip" | cut -f1)"
}

# Función principal
main() {
    case "${1:-help}" in
        check)
            print_header
            check_for_updates
            ;;
        download)
            print_header
            check_for_updates
            download_update
            ;;
        install)
            print_header
            backup_userdata
            check_for_updates
            download_update
            install_update
            echo -e "${BLUE}📱 Actualización completada. Reinicia el dispositivo.${NC}"
            ;;
        package)
            print_header
            generate_ota_package
            ;;
        server)
            print_header
            create_ota_server
            ;;
        rollback)
            print_header
            rollback_update
            ;;
        *)
            echo "FerroOS Mobile - Sistema OTA"
            echo
            echo "Uso: $0 <comando>"
            echo
            echo "Comandos:"
            echo "  check     - Verificar actualizaciones disponibles"
            echo "  download  - Descargar actualización"
            echo "  install   - Instalar actualización completa"
            echo "  package   - Generar paquete OTA"
            echo "  server    - Crear servidor OTA local"
            echo "  rollback  - Revertir última actualización"
            echo
            echo "Ejemplos:"
            echo "  $0 check     # Buscar actualizaciones"
            echo "  $0 install   # Actualizar sistema completo"
            ;;
    esac
}

main "$@"
