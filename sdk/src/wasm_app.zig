const std = @import("std");

// Funciones exportadas por el runtime de Rust
extern fn fos_log(ptr: [*]const u8, len: usize) void;
extern fn fos_clear_screen() void;
extern fn fos_set_color(ptr: [*]const u8, len: usize) void;
extern fn fos_draw_text(ptr: [*]const u8, len: usize) void;
extern fn fos_draw_text_at(text_ptr: [*]const u8, text_len: usize, x: i32, y: i32) void;
extern fn fos_draw_rect(x: i32, y: i32, width: u32, height: u32, filled: bool) void;
extern fn fos_new_line() void;

export fn _start() noreturn {
    const script: []const u8 = @embedFile("assets/app.lua");
    runScript(script);
    std.process.exit(0);
}

fn log(s: []const u8) void {
    fos_log(s.ptr, s.len);
}

fn runScript(src: []const u8) void {
    // Intérprete Lua extendido con funciones gráficas
    var it = std.mem.tokenizeAny(u8, src, "\r\n");
    while (it.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (trimmed.len == 0) continue;
        
        // Ignorar comentarios
        if (std.mem.startsWith(u8, trimmed, "--")) continue;
        
        // Procesar comandos gráficos
        if (processGraphicsCommand(trimmed)) continue;
        
        // Procesar print() tradicional
        if (std.mem.startsWith(u8, trimmed, "print(") and std.mem.endsWith(u8, trimmed, ")")) {
            const inner = trimmed[6 .. trimmed.len - 1];
            const msg = parseStringLiteral(inner) orelse continue;
            log(msg);
            continue;
        }
        
        // Líneas no soportadas - mostrar en log
        log("[Script no soportado]");
        log(trimmed);
    }
}

// Procesar comandos gráficos de Lua
fn processGraphicsCommand(line: []const u8) bool {
    // clear_screen()
    if (std.mem.eql(u8, line, "clear_screen()")) {
        fos_clear_screen();
        return true;
    }
    
    // new_line()
    if (std.mem.eql(u8, line, "new_line()")) {
        fos_new_line();
        return true;
    }
    
    // set_color("color")
    if (std.mem.startsWith(u8, line, "set_color(") and std.mem.endsWith(u8, line, ")")) {
        const inner = line[10 .. line.len - 1];
        if (parseStringLiteral(inner)) |color| {
            fos_set_color(color.ptr, color.len);
            return true;
        }
    }
    
    // draw_text("text")
    if (std.mem.startsWith(u8, line, "draw_text(") and std.mem.endsWith(u8, line, ")")) {
        const inner = line[10 .. line.len - 1];
        if (parseStringLiteral(inner)) |text| {
            fos_draw_text(text.ptr, text.len);
            return true;
        }
    }
    
    // draw_text_at("text", x, y)
    if (std.mem.startsWith(u8, line, "draw_text_at(")) {
        if (parseDrawTextAt(line)) {
            return true;
        }
    }
    
    // draw_rect(x, y, width, height, filled)
    if (std.mem.startsWith(u8, line, "draw_rect(")) {
        if (parseDrawRect(line)) {
            return true;
        }
    }
    
    return false;
}

// Parsear draw_text_at("text", x, y)
fn parseDrawTextAt(line: []const u8) bool {
    // Extraer contenido entre paréntesis
    const start = std.mem.indexOf(u8, line, "(") orelse return false;
    const end = std.mem.lastIndexOf(u8, line, ")") orelse return false;
    if (end <= start) return false;
    
    const params = line[start + 1 .. end];
    
    // Buscar primera coma (después del string)
    var comma1: usize = 0;
    var in_quotes = false;
    
    for (params, 0..) |c, i| {
        if (c == '"') in_quotes = !in_quotes;
        if (c == ',' and !in_quotes) {
            comma1 = i;
            break;
        }
    }
    
    if (comma1 == 0) return false;
    
    // Buscar segunda coma
    const comma2 = std.mem.indexOfPos(u8, params, comma1 + 1, ",") orelse return false;
    
    // Extraer partes
    const text_part = std.mem.trim(u8, params[0..comma1], " \t");
    const x_part = std.mem.trim(u8, params[comma1 + 1..comma2], " \t");
    const y_part = std.mem.trim(u8, params[comma2 + 1..], " \t");
    
    // Parsear texto
    const text = parseStringLiteral(text_part) orelse return false;
    
    // Parsear coordenadas
    const x = std.fmt.parseInt(i32, x_part, 10) catch return false;
    const y = std.fmt.parseInt(i32, y_part, 10) catch return false;
    
    fos_draw_text_at(text.ptr, text.len, x, y);
    return true;
}

// Parsear draw_rect(x, y, width, height, filled)
fn parseDrawRect(line: []const u8) bool {
    const start = std.mem.indexOf(u8, line, "(") orelse return false;
    const end = std.mem.lastIndexOf(u8, line, ")") orelse return false;
    if (end <= start) return false;
    
    const params = line[start + 1 .. end];
    
    // Dividir por comas
    var parts = std.mem.splitScalar(u8, params, ',');
    
    const x_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const y_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const width_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const height_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const filled_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    
    const x = std.fmt.parseInt(i32, x_str, 10) catch return false;
    const y = std.fmt.parseInt(i32, y_str, 10) catch return false;
    const width = std.fmt.parseInt(u32, width_str, 10) catch return false;
    const height = std.fmt.parseInt(u32, height_str, 10) catch return false;
    const filled = std.mem.eql(u8, filled_str, "true");
    
    fos_draw_rect(x, y, width, height, filled);
    return true;
}

fn parseStringLiteral(s: []const u8) ?[]const u8 {
    // Aceptamos "..." sin escapes complejos; si no cumple, devolvemos null
    const t = std.mem.trim(u8, s, " \t");
    if (t.len < 2) return null;
    if (t[0] != '"' or t[t.len - 1] != '"') return null;
    return t[1 .. t.len - 1];
}
