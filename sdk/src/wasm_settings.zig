const std = @import("std");

// Funciones exportadas por el runtime de Rust (Mismas firmas que wasm_app.zig)
extern fn fos_log(ptr: [*]const u8, len: usize) void;
extern fn fos_clear_screen() void;
extern fn fos_set_color(ptr: [*]const u8, len: usize) void;
extern fn fos_draw_text(ptr: [*]const u8, len: usize) void;
extern fn fos_draw_text_at(text_ptr: [*]const u8, text_len: usize, x: i32, y: i32) void;
extern fn fos_draw_rect(x: i32, y: i32, width: u32, height: u32, filled: bool) void;
extern fn fos_new_line() void;

// Entry point
export fn _start() noreturn {
    const script: []const u8 = @embedFile("assets/settings.lua");
    runScript(script);
    std.process.exit(0);
}

fn log(s: []const u8) void {
    fos_log(s.ptr, s.len);
}

fn runScript(src: []const u8) void {
    var it = std.mem.tokenizeAny(u8, src, "\r\n");
    while (it.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (trimmed.len == 0) continue;
        if (std.mem.startsWith(u8, trimmed, "--")) continue;
        if (processGraphicsCommand(trimmed)) continue;
    }
}

fn processGraphicsCommand(line: []const u8) bool {
    if (std.mem.eql(u8, line, "clear_screen()")) { fos_clear_screen(); return true; }
    if (std.mem.startsWith(u8, line, "set_color(")) { 
        if (std.mem.indexOf(u8, line, "black") != null) { fos_set_color("black".ptr, 5); return true; }
        if (std.mem.indexOf(u8, line, "white") != null) { fos_set_color("white".ptr, 5); return true; }
        if (std.mem.indexOf(u8, line, "blue") != null) { fos_set_color("blue".ptr, 4); return true; }
        return true; 
    }
    if (std.mem.startsWith(u8, line, "draw_text_at(")) {
        return parseDrawTextAt(line);
    }
    if (std.mem.startsWith(u8, line, "draw_rect(")) {
        return parseDrawRect(line);
    }
    return false;
}

fn parseDrawTextAt(line: []const u8) bool {
    const start = std.mem.indexOf(u8, line, "(") orelse return false;
    const end = std.mem.lastIndexOf(u8, line, ")") orelse return false;
    if (end <= start) return false;
    const params = line[start + 1 .. end];
    var comma1: usize = 0;
    var in_quotes = false;
    for (params, 0..) |c, i| {
        if (c == '"') in_quotes = !in_quotes;
        if (c == ',' and !in_quotes) { comma1 = i; break; }
    }
    if (comma1 == 0) return false;
    const comma2 = std.mem.indexOfPos(u8, params, comma1 + 1, ",") orelse return false;
    const text_part = std.mem.trim(u8, params[0..comma1], " \t");
    const x_part = std.mem.trim(u8, params[comma1 + 1..comma2], " \t");
    const y_part = std.mem.trim(u8, params[comma2 + 1..], " \t");
    
    // Parse String Literal
    const text = if (text_part.len >= 2 and text_part[0] == '"' and text_part[text_part.len-1] == '"') text_part[1..text_part.len-1] else return false;
    
    const x = std.fmt.parseInt(i32, x_part, 10) catch return false;
    const y = std.fmt.parseInt(i32, y_part, 10) catch return false;
    fos_draw_text_at(text.ptr, text.len, x, y);
    return true;
}

fn parseDrawRect(line: []const u8) bool {
    const start = std.mem.indexOf(u8, line, "(") orelse return false;
    const end = std.mem.lastIndexOf(u8, line, ")") orelse return false;
    if (end <= start) return false;
    const params = line[start + 1 .. end];
    var parts = std.mem.splitScalar(u8, params, ',');
    const x_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const y_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const w_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const h_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    const f_str = std.mem.trim(u8, parts.next() orelse return false, " \t");
    
    const x = std.fmt.parseInt(i32, x_str, 10) catch return false;
    const y = std.fmt.parseInt(i32, y_str, 10) catch return false;
    const w = std.fmt.parseInt(u32, w_str, 10) catch return false;
    const h = std.fmt.parseInt(u32, h_str, 10) catch return false;
    const filled = std.mem.eql(u8, std.mem.trim(u8, f_str, "\""), "true");
    
    fos_draw_rect(x, y, w, h, filled);
    return true;
}
