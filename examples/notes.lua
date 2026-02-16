-- Ejemplo: App de Notas
clear_screen()

-- Fondo
set_color("yellow")
draw_rect(0, 0, 640, 480, "true")

-- Header
set_color("black")
draw_text_at("=== MIS NOTAS ===", 240, 20)

-- Nota 1
set_color("white")
draw_rect(50, 60, 540, 80, "true")
set_color("black")
draw_text_at("Comprar leche", 70, 80)
draw_text_at("Recordatorio para mañana", 70, 105)

-- Nota 2
set_color("white")
draw_rect(50, 160, 540, 80, "true")
set_color("black")
draw_text_at("Reunion a las 3pm", 70, 180)
draw_text_at("Con el equipo de desarrollo", 70, 205)

-- Nota 3
set_color("white")
draw_rect(50, 260, 540, 80, "true")
set_color("black")
draw_text_at("Estudiar Lua", 70, 280)
draw_text_at("Crear apps para FerroOS", 70, 305)

-- Botón agregar
set_color("green")
draw_rect(250, 380, 140, 50, "true")
set_color("white")
draw_text_at("+ Nueva Nota", 270, 400)
