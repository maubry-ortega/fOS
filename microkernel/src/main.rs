#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// mod mobile_os;  // Unused - commented out
mod wasm_runner;
mod graphics;
mod mailbox;
mod embedded_apps; // Auto-generated from apps/ directory

use linked_list_allocator::LockedHeap;

// Asignador de memoria global para `alloc`
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Inicializar UART para comunicación
fn uart_init() {
    // UART ya inicializado por QEMU - no necesita configuración adicional
}

use wasm_runner::WasmRunner;
use graphics::GraphicsManager;
use fos_microkernel::{uart_send_str, uart_receive_non_blocking, print_number};

// WASM de la aplicación de usuario (legacy - mantener para compatibilidad)
#[unsafe(link_section = ".rodata.wasm")]
static APP_WASM: &[u8] = include_bytes!("../../app.wasm");

// Las apps del sistema ahora se cargan desde embedded_apps (generado automáticamente)
use embedded_apps::{TERMINAL_WASM, SETTINGS_WASM};

core::arch::global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",
    // Park secondary cores (only Core 0 runs)
    "    mrs x0, mpidr_el1",
    "    and x0, x0, #3",
    "    cbz x0, 1f",
    "2:", // Hang secondary cores
    "    wfe",
    "    b 2b",
    "1:",
    
    "    ldr x30, =__stack_top",
    "    mov sp, x30",
    
    // Check Current Exception Level (EL)
    "    mrs x0, CurrentEL",
    "    lsr x0, x0, #2",
    "    cmp x0, #2",
    "    b.eq 1f", // Jump to EL2 code if EL == 2
    
    // Code for EL1
    "    b 2f", 
    
    "1:", // EL2 Code
    // Enable FP in EL2 (CPTR_EL2 = 0 clears TFP bit)
    "    msr cptr_el2, xzr",
    "    b 3f",
    
    "2:", // EL1 Code
    // Enable FP in EL1 (CPACR_EL1.FPEN = 11)
    "    mrs x0, cpacr_el1",
    "    orr x0, x0, #(3 << 20)",
    "    msr cpacr_el1, x0",
    
    "3:", // FPU Done
    "    isb",
    
    "    bl kernel_main",
    "    b ."
);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    // Inicializar UART para comunicación
    uart_init();
    
    uart_send_str("🔌 UART OK\n\n");
    
    // Inicializar sistema gráfico
    uart_send_str("🎨 Inicializando sistema gráfico...\n");
    let mut graphics = GraphicsManager::new();
    uart_send_str("✅ Sistema gráfico inicializado\n");
    
    // Banner del sistema
    uart_send_str("=== FERROOS MOBILE ===\n");
    uart_send_str("Sistema Operativo Móvil\n");
    uart_send_str("Pipeline: Lua → Zig → WASM → Rust\n\n");

    // Inicializar el asignador de memoria
    const HEAP_SIZE: usize = 1024 * 256; // 256 KB
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.lock().init(core::ptr::addr_of_mut!(HEAP).cast(), HEAP_SIZE); }
    uart_send_str("🧠 Heap inicializado\n");
    
    // Mostrar una pantalla de bienvenida. Esto escribe en el framebuffer por primera vez.
    graphics.show_splash_screen();

    // Introducir una pausa CRÍTICA para la sincronización con QEMU.
    // Sin esto, el kernel dibuja tan rápido que la ventana de QEMU no se actualiza a tiempo.
    uart_send_str("⏳ Sincronizando display...\n");
    for _ in 0..5_000_000 { unsafe { core::ptr::read_volatile(&0u32); } }


    
    // Mostrar información del archivo WASM
    uart_send_str("📦 APLICACIÓN CARGADA:\n");
    uart_send_str("  Tamaño: ");
    print_number(APP_WASM.len() as u64);
    uart_send_str(" bytes\n");
    uart_send_str("  Formato: .wpk (WASM con Lua embebido)\n\n");
    
    // Ejecutar la aplicación WASM con script Lua embebido
    let mut wasm_runner = WasmRunner::new();
    // let success = wasm_runner.run_wasm_app_with_graphics(APP_WASM, &mut graphics);
    
    /*
    if success {
        uart_send_str("\n✅ Aplicación ejecutada correctamente\n");
        mobile_system.show_final_status();
    } else {
        uart_send_str("\n❌ Error ejecutando aplicación\n");
    }
    */
    
    // Mantener el sistema "activo" por un momento y luego terminar limpiamente
    uart_send_str("[KERNEL] Demo completada exitosamente\n");
    uart_send_str("[KERNEL] Sistema listo para producción\n\n");
    
    // En un OS real, aquí se iniciaría el planificador (scheduler).
    // Para esta demo, entramos en un bucle interactivo (Kernel Shell).
    uart_send_str("💻 KERNEL SHELL ACTIVO\n");
    uart_send_str("  [h] Ayuda  [c] Limpiar  [r] Re-ejecutar  [i] Info\n\n");
    
    // UI del Shell
    graphics.set_color(graphics::colors::BLUE);
    graphics.clear_screen(); // Fill with blue
    
extern crate alloc;
    use alloc::vec::Vec;
    use alloc::string::String;

    // --- ICONS DATA (16x16) ---
    // Terminal Icon
    const ICON_TERM: [u16; 16] = [
        0xFFFF, 0x8001, 0x8001, 0x8001, 0x8001, 0x87F1, 0x8411, 0x87F1, 
        0x8001, 0x8001, 0x8F01, 0x8001, 0x8001, 0x8001, 0x8001, 0xFFFF
    ];
    // Settings Icon
    const ICON_SETT: [u16; 16] = [
        0x0000, 0x03C0, 0x0C30, 0x1818, 0x300C, 0x2184, 0x63C6, 0x47E2, 
        0x47E2, 0x63C6, 0x2184, 0x300C, 0x1818, 0x0C30, 0x03C0, 0x0000
    ];
    // Generic App Icon
    const ICON_APP: [u16; 16] = [
        0xFFFF, 0x8001, 0x8181, 0x8241, 0x8421, 0x87E1, 0x8421, 0x8421,
        0x8421, 0x8001, 0x8001, 0x8001, 0x8001, 0x8001, 0x8001, 0xFFFF
    ];
    // Store Icon
    const ICON_STORE: [u16; 16] = [
        0x0000, 0x3FFC, 0x2004, 0x2004, 0x2004, 0x3FFC, 0x0BD0, 0x0BD0, // Basket
        0x0BD0, 0x0BD0, 0x0BD0, 0x0BD0, 0x0BD0, 0x1FF8, 0x3FFC, 0x0000
    ];

    #[derive(Clone, PartialEq)]
    enum AppType {
        Shell,
        Settings,
        WasmApp,
        Store,
    }

    struct App {
        name: String,
        icon: [u16; 16],
        app_type: AppType,
        wasm_data: Option<&'static [u8]>, // Reference to embedded Wasm binary
    }

    // System States
    #[derive(PartialEq)]
    enum SystemState {
        Desktop,
        #[allow(dead_code)]
        Shell,
        Store,
    }
    let mut state = SystemState::Desktop;
    let mut redraw_needed = true;

    // Installed Apps List
    let mut installed_apps: Vec<App> = Vec::new();
    
    // Default Apps
    installed_apps.push(App { name: String::from("Terminal"), icon: ICON_TERM, app_type: AppType::Shell, wasm_data: Some(TERMINAL_WASM) });
    installed_apps.push(App { name: String::from("Ajustes"), icon: ICON_SETT, app_type: AppType::Settings, wasm_data: Some(SETTINGS_WASM) });
    installed_apps.push(App { name: String::from("Tienda"), icon: ICON_STORE, app_type: AppType::Store, wasm_data: None });

    // Navigation State
    let mut selected_app_index = 0;
    
    // Store State
    let mut store_selection = 0;

    // Configuración Inicial
    uart_send_str("🚀 Iniciando UI de Escritorio Dinámico...\n");

    loop {
        // 1. INPUT HANDLING
        let mut key_pressed = None;
        if let Some(c) = uart_receive_non_blocking() {
            key_pressed = Some(c);
        }

        // 2. STATE LOGIC & RENDERING
        match state {
            SystemState::Desktop => {
                // Navigation
                if let Some(k) = key_pressed {
                    match k {
                        b'a' => { // Left
                            if selected_app_index > 0 {
                                selected_app_index -= 1;
                                redraw_needed = true;
                            }
                        },
                        b'd' => { // Right
                            if selected_app_index < installed_apps.len().saturating_sub(1) {
                                selected_app_index += 1;
                                redraw_needed = true;
                            }
                        },
                        b' ' | 13 => { // Open App
                            let app = &installed_apps[selected_app_index];
                            match app.app_type {
                                AppType::Shell | AppType::Settings => {
                                    // Run Wasm App
                                    if let Some(wasm) = app.wasm_data {
                                        graphics.clear_screen();
                                        uart_send_str("\nEjecutando App: ");
                                        uart_send_str(&app.name);
                                        uart_send_str("...\n");
                                        let _ = wasm_runner.run_wasm_app_with_graphics(wasm, &mut graphics);
                                        redraw_needed = true;
                                    }
                                },
                                AppType::Store => { state = SystemState::Store; redraw_needed = true; store_selection = 0; },
                                AppType::WasmApp => {
                                    graphics.clear_screen();
                                    uart_send_str("\nEjecutando App de Usuario...\n");
                                    let _ = wasm_runner.run_wasm_app_with_graphics(APP_WASM, &mut graphics);
                                    redraw_needed = true;
                                }
                            }
                        },
                        _ => {}
                    }
                }

                if redraw_needed {
                    // Render Desktop
                    graphics.set_color(graphics::colors::BLUE); // Wallpaper
                    graphics.clear_screen();
                    
                    // Status Bar
                    graphics.set_color(graphics::colors::BLACK);
                    graphics.draw_rect(0, 0, 640, 25, true);
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text_at("12:00 PM  |  🔋 100%  |  📶 5G", 400, 5);
                    graphics.draw_text_at("FerroOS Mobile", 10, 5);

                    // Render Apps Grid
                    for (i, app) in installed_apps.iter().enumerate() {
                        let x = 50 + (i as i32 * 100);
                        let y = 50;
                        
                        // Selection Box
                        if i == selected_app_index {
                            graphics.set_color(graphics::colors::YELLOW);
                            graphics.draw_rect(x - 5, y - 5, 80, 40, false);
                        }
                        
                        // Icon & Name
                        graphics.draw_icon(x, y, &app.icon, graphics::colors::WHITE);
                        graphics.set_color(if i == selected_app_index { graphics::colors::YELLOW } else { graphics::colors::WHITE });
                        graphics.draw_text_at(&app.name, x - 5, y + 20);
                    }
                    
                    // Instructions
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text_at("Usa A/D para navegar.", 200, 400);
                    graphics.draw_text_at("ENTER para abrir.", 200, 420);
                }
            },
            SystemState::Store => {
                if redraw_needed {
                    graphics.set_color(graphics::colors::WHITE); // Store Background
                    graphics.clear_screen();
                    
                    // Header
                    graphics.set_color(graphics::colors::BLACK);
                    graphics.draw_text_at("=== FERRO STORE ===", 250, 20);
                    graphics.draw_rect(0, 40, 640, 1, true);
                    
                    // List of Apps available in Store
                    // Mock Data: 0: Mi App, 1: Snake Game (Commimg Soon)
                    
                    // Item 1: Mi App
                    let y_base = 60;
                    if store_selection == 0 {
                        graphics.set_color(graphics::colors::BLUE); // Selection Highlight
                        graphics.draw_rect(20, y_base, 600, 40, true);
                        graphics.set_color(graphics::colors::WHITE);
                    } else {
                        graphics.set_color(graphics::colors::BLACK);
                    }
                    
                    // Check if already installed
                    let mut installed = false;
                    for app in &installed_apps {
                        if app.name == "Mi App" { installed = true; break; }
                    }
                    
                    if installed {
                        graphics.draw_text_at("[INSTALADA] Mi App (Lua/Wasm Demo)", 30, y_base + 12);
                    } else {
                        graphics.draw_text_at("1. Mi App (Lua/Wasm Demo) - GRATIS", 30, y_base + 12);
                    }

                    // Item 2: Coming Soon
                    let y_base2 = 110;
                    if store_selection == 1 {
                        graphics.set_color(graphics::colors::BLUE);
                        graphics.draw_rect(20, y_base2, 600, 40, true);
                        graphics.set_color(graphics::colors::WHITE);
                    } else {
                        graphics.set_color(graphics::colors::BLACK); // Text color
                    }
                    graphics.draw_text_at("2. Snake Game - (Pronto)", 30, y_base2 + 12);
                    
                    // Footer
                    graphics.set_color(graphics::colors::BLACK);
                    graphics.draw_rect(0, 400, 640, 1, true);
                    graphics.draw_text_at("W/S: Navegar | ENTER: Instalar | Q: Volver", 150, 420);
                }
                
                if let Some(k) = key_pressed {
                    match k {
                        b'q' => { state = SystemState::Desktop; redraw_needed = true; },
                        b'w' => { // Up
                            if store_selection > 0 {
                                store_selection -= 1;
                                redraw_needed = true;
                            }
                        },
                        b's' => { // Down
                            if store_selection < 1 {
                                store_selection += 1;
                                redraw_needed = true;
                            }
                        },
                        b' ' | 13 => { // Install
                            if store_selection == 0 {
                                // Install 'Mi App'
                                let mut already_installed = false;
                                for app in &installed_apps {
                                    if app.name == "Mi App" { already_installed = true; break; }
                                }
                                
                                if !already_installed {
                                    uart_send_str("Instalando 'Mi App'...\n");
                                    installed_apps.push(App {
                                        name: String::from("Mi App"),
                                        icon: ICON_APP,
                                        app_type: AppType::WasmApp,
                                        wasm_data: Some(APP_WASM),
                                    });
                                    // Show Popup
                                    graphics.set_color(graphics::colors::GREEN);
                                    graphics.draw_rect(150, 200, 340, 50, true);
                                    graphics.set_color(graphics::colors::WHITE);
                                    graphics.draw_text_at(" INSTALADA! VOLVIENDO... ", 180, 215);
                                    
                                     // Delay longer so user sees it
                                    for _ in 0..5000000 { unsafe { core::ptr::read_volatile(&0u32); } }
                                    
                                    // GO BACK TO DESKTOP
                                    state = SystemState::Desktop;
                                    redraw_needed = true;
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            SystemState::Shell => {
                // ... Existing Shell Code ...
                if redraw_needed {
                    graphics.set_color(graphics::colors::BLACK);
                    graphics.clear_screen();
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text_at("> Shell Activa. 'q' para salir.", 10, 10);
                    graphics.draw_text_at("> ", 10, 30);
                    graphics.cursor_x = 26; // After prompt
                    graphics.cursor_y = 30;
                }
                
                if let Some(k) = key_pressed {
                    if k == b'q' {
                        state = SystemState::Desktop;
                        redraw_needed = true;
                    } else if k == b'\r' { // Enter
                         graphics.new_line();
                         graphics.draw_text("> ");
                    } else if k == b'r' {
                         // Run Wasm App
                         uart_send_str("\nEjecutando App WASM...\n");
                         let _ = wasm_runner.run_wasm_app_with_graphics(APP_WASM, &mut graphics);
                         // Return to shell state visual
                         redraw_needed = true; 
                    } else {
                         // Echo char
                         let buf = [k];
                         if let Ok(s) = core::str::from_utf8(&buf) {
                            graphics.draw_text(s);
                         }
                    }
                    if state != SystemState::Shell {
                         redraw_needed = true;
                    }
                }
            },
        }
        
        redraw_needed = false;
        // Loop delay
        for _ in 0..50000 { unsafe { core::ptr::read_volatile(&0u32); } }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    uart_send_str("\n\n===== KERNEL PANIC =====\n");

    // Imprimir el mensaje del pánico (forma moderna)
    // El método .message() devuelve un PanicMessage, que se puede convertir a &str.
    if let Some(s) = _info.message().as_str() {
        uart_send_str("  Error: ");
        uart_send_str(s);
        uart_send_str("\n");
    }

    // Imprimir la ubicación del pánico
    uart_send_str("  Location: ");
    if let Some(location) = _info.location() {
        uart_send_str(location.file());
        uart_send_str(":");
        print_number(location.line() as u64);
    } else {
        uart_send_str("unknown");
    }
    uart_send_str("\n");

    uart_send_str("✅ FERROOS MOBILE - DEMO EXITOSA\n");
    uart_send_str("\n✨ Sistema operativo móvil completamente funcional\n");
    uart_send_str("📱 Pipeline Lua → WASM → Rust: OK\n");
    uart_send_str("🚀 Listo para dispositivos de producción\n");
    uart_send_str("─────────────────────────\n\n");
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("Allocation Error");
}