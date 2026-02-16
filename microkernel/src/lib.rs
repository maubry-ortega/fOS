#![no_std]

// Microkernel móvil minimalista



// Dirección UART PL011 para Raspberry Pi 3
const UART0_BASE: usize = 0x3F20_1000;  // PL011 UART  
const UART0_DR: *mut u32 = (UART0_BASE + 0x00) as *mut u32; // Data Register
const UART0_FR: *mut u32 = (UART0_BASE + 0x18) as *mut u32; // Flag Register

// Enviar un byte por UART PL011
pub fn uart_send(byte: u8) {
    unsafe {
        // Esperar que el FIFO de transmisión no esté lleno (bit 5 = TXFF)
        while UART0_FR.read_volatile() & (1 << 5) != 0 {}
        // Escribir el byte al registro de datos
        UART0_DR.write_volatile(byte as u32);
    }
}

// Recibir un byte por UART (bloqueante)
pub fn uart_receive() -> u8 {
    unsafe {
        // Esperar que el FIFO de recepción no esté vacío (bit 4 = RXFE)
        while UART0_FR.read_volatile() & (1 << 4) != 0 {}
        // Leer el byte del registro de datos
        UART0_DR.read_volatile() as u8
    }
}

// Recibir un byte por UART (no bloqueante)
pub fn uart_receive_non_blocking() -> Option<u8> {
    unsafe {
        // Comprobar si el FIFO de recepción está vacío (bit 4 = RXFE)
        if UART0_FR.read_volatile() & (1 << 4) != 0 {
            None
        } else {
            Some(UART0_DR.read_volatile() as u8)
        }
    }
}

// Enviar string por UART
pub fn uart_send_str(s: &str) {
    for b in s.as_bytes() { 
        uart_send(*b); 
    }
}

pub fn print_number(mut n: u64) {
    if n == 0 {
        uart_send_str("0");
        return;
    }
    
    let mut buf = [0u8; 20];
    let mut i = 0;
    
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    
    while i > 0 {
        i -= 1;
        uart_send(buf[i]);
    }
}