//! Runner WASM para aplicaciones móviles de FerroOS
//! 
//! Procesa aplicaciones .wpk que contienen scripts Lua embebidos en WASM

use fos_microkernel::{uart_send_str, print_number};
use crate::graphics::{GraphicsManager, colors};

/// Runtime WASM que extrae y ejecuta scripts Lua
pub struct WasmRunner {
    _memory: [u8; 32 * 1024], // 32KB para apps simples (prefijo _ para evitar warning)
}

impl WasmRunner {
    pub fn new() -> Self {
        Self {
            _memory: [0; 32 * 1024],
        }
    }

    /// Ejecutar aplicación WASM con script Lua embebido y soporte gráfico
    pub fn run_wasm_app_with_graphics(&mut self, wasm_data: &[u8], graphics: &mut GraphicsManager) -> bool {
        uart_send_str("📱 EJECUTANDO APLICACIÓN MÓVIL (.wpk)\n");
        
        // Configurar contexto gráfico global
        set_graphics_context(graphics);
        
        // Verificar magic number WASM
        if !self.is_valid_wasm(wasm_data) {
            uart_send_str("❌ Formato WASM inválido\n");
            return false;
        }

        uart_send_str("✅ WASM válido detectado\n");
        
        // Extraer script Lua del WASM
        if let Some(lua_script) = self.extract_lua_from_wasm(wasm_data) {
            uart_send_str("📄 Script Lua encontrado, ejecutando con gráficos...\n\n");
            
            // Procesar script Lua con comandos gráficos
            self.execute_lua_script_graphics(lua_script);
            
            uart_send_str("\n✅ Aplicación gráfica ejecutada exitosamente\n");
            true
        } else {
            uart_send_str("❌ No se encontró script Lua en el WASM\n");
            false
        }
    }
    
    /// Verificar si los datos son un WASM válido
    fn is_valid_wasm(&self, data: &[u8]) -> bool {
        data.len() >= 4 && 
        data[0] == 0x00 && data[1] == 0x61 && data[2] == 0x73 && data[3] == 0x6d
    }

    /// Extraer script Lua del binario WASM
    fn extract_lua_from_wasm<'a>(&self, wasm_data: &'a [u8]) -> Option<&'a str> {
        // Buscar el patrón "print(" en el WASM para encontrar el script Lua
        for i in 0..(wasm_data.len().saturating_sub(6)) {
            if &wasm_data[i..i+6] == b"print(" {
                // Encontramos el inicio del script Lua embebido
                return self.find_lua_script_at(wasm_data, i);
            }
        }
        None
    }

    /// Buscar el script Lua completo a partir de una posición
    fn find_lua_script_at<'a>(&self, wasm_data: &'a [u8], start: usize) -> Option<&'a str> {
        // Buscar hacia atrás para encontrar el inicio del script
        let mut script_start = start;
        while script_start > 0 {
            if wasm_data[script_start - 1] == 0 {
                break;
            }
            script_start -= 1;
        }

        // Buscar el final del script (primer byte nulo después del script)
        let mut script_end = start;
        while script_end < wasm_data.len() {
            if wasm_data[script_end] == 0 && script_end > start + 100 { // Al menos 100 chars
                break;
            }
            script_end += 1;
        }

        // Convertir a string si es válido UTF-8
        if let Ok(script) = core::str::from_utf8(&wasm_data[script_start..script_end]) {
            // Verificar que realmente parece un script Lua
            if script.contains("print(") && script.len() > 20 {
                return Some(script);
            }
        }
        
        None
    }

    /// Ejecutar script Lua con funciones gráficas (versión simplificada)
    fn execute_lua_script_graphics(&self, script: &str) {
        uart_send_str("🎨 INTERPRETANDO SCRIPT LUA CON GRÁFICOS\n");

        let graphics_option = get_graphics_context();
        if graphics_option.is_none() {
            uart_send_str("❌ Error: Contexto gráfico no disponible.\n");
            return;
        }
        let graphics = graphics_option.unwrap();

        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("--") {
                continue;
            }

            self.parse_and_execute_lua_command(line, graphics);

            // Pequeña pausa para ver el dibujo progresivo
            for _ in 0..200000 {
                unsafe { core::ptr::read_volatile(&0u32) };
            }
        }

        uart_send_str("✅ Script Lua interpretado completamente\n");
    }

    /// Parsea y ejecuta un comando gráfico de Lua
    fn parse_and_execute_lua_command(&self, line: &str, graphics: &mut GraphicsManager) {
        if let Some(params) = self.get_params(line, "clear_screen") {
            if params.is_empty() {
                graphics.clear_screen();
            }
        } else if let Some(params) = self.get_params(line, "set_color") {
            if let Some(color_str) = params.get(0) {
                graphics.set_color(parse_color(color_str));
            }
        } else if let Some(params) = self.get_params(line, "draw_text") {
            if let Some(text) = params.get(0) {
                graphics.draw_text(text);
            }
        } else if let Some(params) = self.get_params(line, "draw_text_at") {
            if let (Some(text), Some(x_str), Some(y_str)) = (params.get(0), params.get(1), params.get(2)) {
                if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                    graphics.draw_text_at(text, x, y);
                }
            }
        } else if let Some(params) = self.get_params(line, "draw_rect") {
            if let (Some(x_str), Some(y_str), Some(w_str), Some(h_str), Some(f_str)) =
                (params.get(0), params.get(1), params.get(2), params.get(3), params.get(4))
            {
                if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                    x_str.parse::<i32>(),
                    y_str.parse::<i32>(),
                    w_str.parse::<u32>(),
                    h_str.parse::<u32>(),
                ) {
                    let filled = f_str == &"true";
                    graphics.draw_rect(x, y, w, h, filled);
                }
            }
        }
    }

    /// Extrae los parámetros de una llamada a función Lua simple
    /// Ejemplo: "draw_text("Hola", 10, 20)" -> ["Hola", "10", "20"]
    fn get_params<'a>(&self, line: &'a str, func_name: &str) -> Option<alloc::vec::Vec<&'a str>> {
        if line.starts_with(func_name) && line.contains('(') && line.ends_with(')') {
            let start = line.find('(')? + 1;
            let end = line.rfind(')')?;
            if start >= end {
                return Some(alloc::vec![]);
            }
            let params_str = &line[start..end];
            let params = params_str
                .split(',')
                .map(|p| p.trim().trim_matches('"'))
                .collect();
            return Some(params);
        }
        None
    }
}

// Se necesita la caja `alloc` para usar `Vec`
extern crate alloc;

// Variables globales para el contexto gráfico
static mut GRAPHICS_CONTEXT: Option<*mut GraphicsManager> = None;

/// Inicializar contexto gráfico global
pub fn set_graphics_context(graphics: &mut GraphicsManager) {
    unsafe {
        GRAPHICS_CONTEXT = Some(graphics as *mut GraphicsManager);
    }
}

/// Obtener contexto gráfico global
fn get_graphics_context() -> Option<&'static mut GraphicsManager> {
    unsafe {
        GRAPHICS_CONTEXT.and_then(|ptr| ptr.as_mut())
    }
}

/// Helper para convertir string de color
fn parse_color(color_str: &str) -> embedded_graphics::pixelcolor::Rgb888 {
    match color_str {
        "black" => colors::BLACK,
        "white" => colors::WHITE,
        "red" => colors::RED,
        "green" => colors::GREEN,
        "blue" => colors::BLUE,
        "yellow" => colors::YELLOW,
        "purple" => colors::PURPLE,
        "orange" => colors::ORANGE,
        "cyan" => embedded_graphics::pixelcolor::Rgb888::new(0, 255, 255),
        _ => colors::WHITE,
    }
}

// ===== FUNCIONES EXPORTADAS PARA EL WASM =====

/// Implementación de la función fos_log que espera el WASM
#[unsafe(no_mangle)]
pub extern "C" fn fos_log(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        if let Ok(s) = core::str::from_utf8(slice) {
            uart_send_str("📱 ");
            uart_send_str(s);
            uart_send_str("\n");
        }
    }
}

/// Limpiar pantalla
#[unsafe(no_mangle)]
pub extern "C" fn fos_clear_screen() {
    if let Some(graphics) = get_graphics_context() {
        graphics.clear_screen();
    }
}

/// Establecer color actual
#[unsafe(no_mangle)]
pub extern "C" fn fos_set_color(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    
    unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        if let Ok(color_str) = core::str::from_utf8(slice) {
            let color = parse_color(color_str);
            if let Some(graphics) = get_graphics_context() {
                graphics.set_color(color);
            }
        }
    }
}

/// Dibujar texto en la posición actual del cursor
#[unsafe(no_mangle)]
pub extern "C" fn fos_draw_text(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    
    unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        if let Ok(text) = core::str::from_utf8(slice) {
            if let Some(graphics) = get_graphics_context() {
                graphics.draw_text(text);
            }
        }
    }
}

/// Dibujar texto en posición específica
#[unsafe(no_mangle)]
pub extern "C" fn fos_draw_text_at(text_ptr: *const u8, text_len: usize, x: i32, y: i32) {
    if text_ptr.is_null() || text_len == 0 {
        return;
    }
    
    unsafe {
        let slice = core::slice::from_raw_parts(text_ptr, text_len);
        if let Ok(text) = core::str::from_utf8(slice) {
            if let Some(graphics) = get_graphics_context() {
                graphics.draw_text_at(text, x, y);
            }
        }
    }
}

/// Dibujar rectángulo
#[unsafe(no_mangle)]
pub extern "C" fn fos_draw_rect(x: i32, y: i32, width: u32, height: u32, filled: bool) {
    if let Some(graphics) = get_graphics_context() {
        graphics.draw_rect(x, y, width, height, filled);
    }
}

/// Nueva línea
#[unsafe(no_mangle)]
pub extern "C" fn fos_new_line() {
    if let Some(graphics) = get_graphics_context() {
        graphics.new_line();
    }
}
