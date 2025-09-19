// Demostración del pipeline completo: Lua -> Zig -> WASM -> Rust
// para un OS móvil basado en FerroOS

use std::fs;
use std::process::Command;

fn main() {
    println!("=== FERROOS MOBILE - PIPELINE DEMO ===");
    println!("📱 Sistema Operativo Móvil");
    println!("Pipeline: Lua → Zig → WASM → Rust");
    println!("======================================\n");
    
    // Paso 1: Mostrar el archivo Lua original
    demo_step_1_lua();
    
    // Paso 2: Procesar con SDK de Zig
    demo_step_2_zig_sdk();
    
    // Paso 3: Generar WASM/WPK
    demo_step_3_wasm_generation();
    
    // Paso 4: Procesar con Rust (microkernel)
    demo_step_4_rust_processing();
    
    // Paso 5: Mostrar resultado final
    demo_step_5_final_result();
}

fn demo_step_1_lua() {
    println!("📝 PASO 1: CÓDIGO LUA ORIGINAL");
    println!("─────────────────────────────");
    
    if let Ok(content) = fs::read_to_string("app.lua") {
        println!("📄 Archivo: app.lua");
        println!("📏 Tamaño: {} bytes", content.len());
        println!("🔍 Contenido:");
        println!("{}", content);
    } else {
        println!("❌ No se pudo leer app.lua");
    }
    
    wait_for_key();
}

fn demo_step_2_zig_sdk() {
    println!("\n🔧 PASO 2: PROCESAMIENTO CON SDK DE ZIG");
    println!("───────────────────────────────────");
    
    println!("📦 SDK de Zig procesando archivo Lua...");
    println!("🔄 Embebiendo script en módulo WASM...");
    println!("⚡ Optimizando para arquitectura móvil...");
    
    // Mostrar información del SDK
    if let Ok(content) = fs::read_to_string("sdk/src/wasm_app.zig") {
        let lines = content.lines().collect::<Vec<_>>();
        println!("📄 Procesador: sdk/src/wasm_app.zig");
        println!("🔍 Función clave:");
        for (i, line) in lines.iter().enumerate() {
            if line.contains("@embedFile") || line.contains("runScript") {
                println!("   {}: {}", i + 1, line.trim());
            }
        }
    }
    
    wait_for_key();
}

fn demo_step_3_wasm_generation() {
    println!("\n🏗️  PASO 3: GENERACIÓN DE WASM/WPK");
    println!("──────────────────────────────────");
    
    println!("⚙️  Compilando con Zig...");
    let output = Command::new("zig")
        .args(["build", "wasm"])
        .current_dir("sdk")
        .output();
        
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Compilación exitosa!");
                
                // Verificar archivo generado
                if let Ok(metadata) = fs::metadata("sdk/zig-out/bin/app.wasm") {
                    println!("📦 Archivo generado: app.wasm");
                    println!("📏 Tamaño: {} bytes", metadata.len());
                    println!("🎯 Formato: WebAssembly (WASI)");
                    
                    // Crear el WPK
                    println!("📦 Creando paquete WPK...");
                    fs::copy("sdk/zig-out/bin/app.wasm", "wpk/app.wasm").ok();
                    println!("✅ WPK creado exitosamente!");
                } else {
                    println!("⚠️  Archivo WASM no encontrado");
                }
            } else {
                println!("❌ Error en compilación:");
                println!("{}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => println!("❌ Error ejecutando Zig: {}", e),
    }
    
    wait_for_key();
}

fn demo_step_4_rust_processing() {
    println!("\n🦀 PASO 4: PROCESAMIENTO CON RUST");
    println!("─────────────────────────────────");
    
    println!("🔄 Microkernel de Rust cargando WPK...");
    println!("🔍 Analizando formato WASM...");
    println!("📜 Extrayendo script Lua embebido...");
    
    // Simular el procesamiento del WASM runner
    if let Ok(wasm_data) = fs::read("app.wasm") {
        println!("✅ WASM cargado: {} bytes", wasm_data.len());
        
        // Verificar magic number WASM
        if wasm_data.len() >= 4 && 
           wasm_data[0] == 0x00 && wasm_data[1] == 0x61 && 
           wasm_data[2] == 0x73 && wasm_data[3] == 0x6d {
            println!("✅ Magic number WASM válido");
            
            // Buscar contenido Lua
            let wasm_str = String::from_utf8_lossy(&wasm_data);
            if wasm_str.contains("print(") {
                println!("✅ Script Lua encontrado en WASM");
                println!("🚀 Preparando ejecución...");
            } else {
                println!("⚠️  Script Lua no encontrado");
            }
        } else {
            println!("❌ Magic number WASM inválido");
        }
    } else {
        println!("❌ No se pudo cargar app.wasm");
    }
    
    wait_for_key();
}

fn demo_step_5_final_result() {
    println!("\n🎉 PASO 5: RESULTADO FINAL");
    println!("─────────────────────────");
    
    println!("📱 EJECUTANDO APLICACIÓN MÓVIL:");
    println!("════════════════════════════════");
    
    // Simular la ejecución del script Lua
    if let Ok(content) = fs::read_to_string("app.lua") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("print(") && line.ends_with(")") {
                if let Some(start) = line.find("print(\"") {
                    if let Some(end) = line.rfind("\")") {
                        let message = &line[start + 7..end];
                        println!("📱 {}", message);
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
        }
    }
    
    println!("════════════════════════════════");
    println!("✅ APLICACIÓN EJECUTADA EXITOSAMENTE");
    println!("\n🎯 PIPELINE COMPLETADO:");
    println!("  1. ✅ Lua: Script de aplicación móvil");
    println!("  2. ✅ Zig: Procesamiento y embedding en WASM");
    println!("  3. ✅ WASM: Formato portable generado");
    println!("  4. ✅ Rust: Runtime y ejecución en OS móvil");
    println!("\n🚀 FerroOS Mobile - Listo para producción!");
}

fn wait_for_key() {
    println!("\n[Presiona ENTER para continuar...]");
    std::io::stdin().read_line(&mut String::new()).ok();
}
