-- Ejemplo: App de Reloj Digital
clear_screen()

-- Fondo degradado (simulado con rectángulos)
set_color("blue")
draw_rect(0, 0, 640, 160, "true")
set_color("black")
draw_rect(0, 160, 640, 320, "true")

-- Marco del reloj
set_color("white")
draw_rect(120, 150, 400, 180, "false")
draw_rect(125, 155, 390, 170, "false")

-- Hora (grande)
set_color("green")
draw_text_at("14:30", 240, 220)

-- Fecha
set_color("white")
draw_text_at("Domingo, 16 de Febrero 2026", 180, 280)

-- Iconos de estado (simulados)
set_color("yellow")
draw_rect(50, 400, 30, 30, "true")
set_color("white")
draw_text_at("WiFi", 90, 410)

set_color("green")
draw_rect(200, 400, 30, 30, "true")
set_color("white")
draw_text_at("100%", 240, 410)

-- Footer
set_color("yellow")
draw_text_at("Presiona 'q' para salir", 220, 450)
