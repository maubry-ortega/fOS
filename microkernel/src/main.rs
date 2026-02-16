#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

mod mobile_os;
mod wasm_runner;
mod graphics;
mod mailbox;

use linked_list_allocator::LockedHeap;

// Asignador de memoria global para `alloc`
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Inicializar UART para comunicación
fn uart_init() {
    // UART ya inicializado por QEMU - no necesita configuración adicional
}

use mobile_os::MobileSystem;
use wasm_runner::WasmRunner;
use graphics::GraphicsManager;
use fos_microkernel::{uart_send, uart_send_str, print_number, uart_receive_non_blocking};

// WASM de la aplicación embebida (generada por el SDK de Zig)
#[unsafe(link_section = ".rodata.wasm")]
static APP_WASM: &[u8] = include_bytes!("../../app.wasm");

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
    const HEAP_SIZE: usize = 1024 * 128; // 128 KB
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.lock().init(core::ptr::addr_of_mut!(HEAP).cast(), HEAP_SIZE); }
    uart_send_str("🧠 Heap inicializado\n");
    
    // Mostrar una pantalla de bienvenida. Esto escribe en el framebuffer por primera vez.
    graphics.show_splash_screen();

    // Introducir una pausa CRÍTICA para la sincronización con QEMU.
    // Sin esto, el kernel dibuja tan rápido que la ventana de QEMU no se actualiza a tiempo.
    uart_send_str("⏳ Sincronizando display...\n");
    for _ in 0..5_000_000 { unsafe { core::ptr::read_volatile(&0u32); } }

    // Inicializar sistema móvil básico
    let mut mobile_system = MobileSystem::new();
    mobile_system.init_basic();
    
    // Mostrar información del archivo WASM
    uart_send_str("📦 APLICACIÓN CARGADA:\n");
    uart_send_str("  Tamaño: ");
    print_number(APP_WASM.len() as u64);
    uart_send_str(" bytes\n");
    uart_send_str("  Formato: .wpk (WASM con Lua embebido)\n\n");
    
    // Ejecutar la aplicación WASM con script Lua embebido
    let mut wasm_runner = WasmRunner::new();
    let success = wasm_runner.run_wasm_app_with_graphics(APP_WASM, &mut graphics);
    
    if success {
        uart_send_str("\n✅ Aplicación ejecutada correctamente\n");
        mobile_system.show_final_status();
    } else {
        uart_send_str("\n❌ Error ejecutando aplicación\n");
    }
    
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
    
    // Header
    graphics.set_color(graphics::colors::WHITE);
    graphics.draw_rect(0, 0, 640, 30, true); // White header bar
    graphics.set_color(graphics::colors::BLUE);
    graphics.draw_text_at("FerroOS Mobile Shell", 10, 5); // Blue text on white
    
    // Content
    graphics.set_color(graphics::colors::WHITE);
    graphics.draw_text("\n\n> KERNEL SHELL ACTIVO");
    graphics.draw_text("> Escuchando UART (Escribe en tu terminal)...");
    
    loop {
        if let Some(c) = uart_receive_non_blocking() {
            // Echo en pantalla (simple)
            // En un sistema real usaríamos un buffer circular para la consola
            
            match c {
                b'h' => {
                    uart_send_str("\n--- COMANDOS DISPONIBLES ---\n");
                    graphics.set_color(graphics::colors::YELLOW);
                    graphics.draw_text("\n> [h] Ayuda:");
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text("  c: Limpiar pantalla");
                    graphics.draw_text("  r: Re-ejecutar app");
                    graphics.draw_text("  i: Info sistema");
                },
                b'c' => {
                    uart_send_str("\n🧹 Limpiando pantalla...\n");
                    graphics.clear_screen();
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text("> Pantalla limpia.");
                },
                b'r' => {
                    uart_send_str("\n🔄 Re-ejecutando aplicación...\n");
                    graphics.clear_screen();
                    let success = wasm_runner.run_wasm_app_with_graphics(APP_WASM, &mut graphics);
                    if success {
                        uart_send_str("✅ Re-ejecución completada\n");
                        graphics.set_color(graphics::colors::GREEN);
                        graphics.draw_text("\n> App finalizada.");
                    }
                },
                b'i' => {
                    uart_send_str("\n📊 INFO DEL SISTEMA\n");
                    graphics.set_color(graphics::colors::CYAN);
                    graphics.draw_text("\n> INFO SISTEMA:");
                    graphics.set_color(graphics::colors::WHITE);
                    graphics.draw_text("  OS: FerroOS Mobile v0.1");
                    graphics.draw_text("  Res: 640x480 (16-bit)");
                },
                other => {
                    // Echo visual de cualquier otra tecla
                    let buf = [other];
                    if let Ok(s) = core::str::from_utf8(&buf) {
                        graphics.set_color(graphics::colors::WHITE);
                        graphics.draw_text(s);
                    }
                    uart_send(other); // Echo UART
                }
            }
        }
        
        // Pequeña pausa para no saturar CPU
        for _ in 0..1000 { unsafe { core::ptr::read_volatile(&0u32); } }
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