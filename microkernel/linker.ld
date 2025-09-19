ENTRY(_start)

MEMORY
{
    /* Para Raspberry Pi 4, empezamos en 0x80000 */
    ram (rwx) : ORIGIN = 0x80000, LENGTH = 128M
}

SECTIONS
{
    . = 0x80000;
    
    .text : {
        *(.text._start)
        *(.text*)
    } > ram
    
    .rodata : {
        *(.rodata*)
        . = ALIGN(8);
        
        /* Sección especial para el WASM embebido */
        __wasm_start = .;
        *(.rodata.wasm*)
        __wasm_end = .;
    } > ram
    
    .data : {
        *(.data*)
    } > ram
    
    .bss : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > ram
    
    /* Stack pointer inicial - al final de la memoria */
    __stack_top = ORIGIN(ram) + LENGTH(ram);
}
