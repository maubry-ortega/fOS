-- Ejemplo: Hola Mundo para FerroOS
clear_screen()
set_color("blue")
draw_rect(0, 0, 640, 480, "true")
set_color("white")
draw_text_at("¡Hola FerroOS!", 200, 200)
draw_text_at("Mi primera app", 200, 230)
draw_text_at("Presiona 'q' para salir", 180, 400)
