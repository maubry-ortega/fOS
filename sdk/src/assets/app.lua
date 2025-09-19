-- FerroOS Mobile App - UI Gráfica en Lua
-- Pipeline: Lua → Zig → WASM → Rust

-- Limpiar pantalla y configurar
clear_screen()
set_color("blue")
draw_text_at("FerroOS Mobile", 300, 50)

-- Título de la aplicación
set_color("white")
draw_text_at("📱 Messenger Pro v2.1", 280, 100)

-- Barra de estado
set_color("green")
draw_rect(50, 140, 700, 3, true)

-- Información de conexión
set_color("yellow")
draw_text("🚀 Iniciando aplicación...")
draw_text("✓ Conectando a servidor...")
set_color("green")
draw_text("✓ Verificando permisos...")
draw_text("✓ Cargando contactos...")

-- Separador
set_color("purple")
draw_rect(100, 280, 600, 2, true)

-- Mensajes recientes
set_color("cyan")
draw_text_at("💬 Mensajes recientes:", 100, 300)

set_color("white")
draw_text("  • María: ¿Vienes a la reunión?")
draw_text("  • Luis: ¡El proyecto quedó genial!")
draw_text("  • Ana: Gracias por la ayuda")

-- Notificaciones
set_color("orange")
draw_text_at("🔔 3 notificaciones pendientes", 100, 450)

-- Estado del sistema
set_color("green")
draw_text("⚡ Ahorro de batería: ACTIVO")

-- Marco de la aplicación
set_color("blue")
draw_rect(40, 40, 720, 520, false)

-- Botón de estado
set_color("green")
draw_rect(300, 500, 200, 40, true)
set_color("black")
draw_text_at("✅ APLICACIÓN LISTA", 320, 510)


