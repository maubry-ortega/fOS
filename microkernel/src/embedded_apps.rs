// Auto-generated file - DO NOT EDIT MANUALLY
// Generated from apps/ directory

#[unsafe(link_section = ".rodata.wasm_settings")]
pub static SETTINGS_WASM: &[u8] = include_bytes!("../../apps/settings.wasm");

#[unsafe(link_section = ".rodata.wasm_terminal")]
pub static TERMINAL_WASM: &[u8] = include_bytes!("../../apps/terminal.wasm");

// App manifest loaded at compile time
pub const APP_MANIFEST: &str = include_str!("../../apps/manifest.json");
