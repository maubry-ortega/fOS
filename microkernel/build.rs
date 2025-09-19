fn main() {
    // Build script simplificado - ya no necesitamos wasm3
    // El runtime de WASM se manejará directamente desde Rust usando wasmtime o similar
    println!("cargo:rerun-if-changed=build.rs");
}
