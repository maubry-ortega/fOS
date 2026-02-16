//! FerroOS Mobile - Módulo de gráficos embebido
//! Sistema de gráficos básico para mostrar UI en pantalla

use embedded_graphics::{mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder}, pixelcolor::Rgb888, prelude::*, primitives::{Rectangle, PrimitiveStyleBuilder}, text::{Baseline, Text}};

use crate::mailbox;
use crate::uart_send_str;
use fos_microkernel::print_number;

/// Resolución de pantalla por defecto (Safe Mode para QEMU)
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

/// Colores básicos para la UI
pub mod colors {
    use embedded_graphics::pixelcolor::Rgb888;
    
    pub const BLACK: Rgb888 = Rgb888::new(0, 0, 0);
    pub const WHITE: Rgb888 = Rgb888::new(255, 255, 255);
    pub const BLUE: Rgb888 = Rgb888::new(0, 100, 255);
    pub const GREEN: Rgb888 = Rgb888::new(0, 255, 0);
    pub const RED: Rgb888 = Rgb888::new(255, 0, 0);
    pub const YELLOW: Rgb888 = Rgb888::new(255, 255, 0);
    pub const PURPLE: Rgb888 = Rgb888::new(128, 0, 128);
    pub const ORANGE: Rgb888 = Rgb888::new(255, 165, 0);
    pub const CYAN: Rgb888 = Rgb888::new(0, 255, 255);
}

/// Framebuffer virtual para renderizar gráficos
pub struct FrameBuffer {
    pixels: *mut u8,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
    pitch: u32,
}

impl FrameBuffer {
    /// Crear nuevo framebuffer usando la dirección y el pitch obtenidos del mailbox
    pub fn new(framebuffer_address: u32, pitch: u32) -> Self {
        // Guess depth from pitch
        let bytes_per_pixel = if pitch == SCREEN_WIDTH * 2 { 2 } else { 4 };
        
        Self {
            pixels: framebuffer_address as *mut u8,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            bytes_per_pixel,
            pitch,
        }
    }
    
    /// Limpiar pantalla con un color
    pub fn clear(&mut self, color: Rgb888) {
        let (r, g, b) = (color.r(), color.g(), color.b());
        
        // Pre-calculate 16-bit color
        let r5 = (r >> 3) as u16;
        let g6 = (g >> 2) as u16;
        let b5 = (b >> 3) as u16;
        let rgb565 = (r5 << 11) | (g6 << 5) | b5;
        
        unsafe {
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.bytes_per_pixel == 2 {
                        let offset = (y * self.pitch + x * 2) as usize;
                        *(self.pixels.add(offset) as *mut u16) = rgb565;
                    } else {
                        let offset = (y * self.pitch + x * 4) as usize;
                        *self.pixels.add(offset) = b; // Blue (BGRA)
                        *self.pixels.add(offset + 1) = g; // Green
                        *self.pixels.add(offset + 2) = r; // Red
                        *self.pixels.add(offset + 3) = 255; // Alpha
                    }
                }
            }
        }
    }
    
}

impl DrawTarget for FrameBuffer {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let x = coord.x as u32;
            let y = coord.y as u32;
            
            if x < self.width && y < self.height {
                let index = (y * self.pitch + x * self.bytes_per_pixel) as usize;
                
                unsafe {
                    *self.pixels.add(index) = color.r();     // Red
                    *self.pixels.add(index + 1) = color.g(); // Green
                    *self.pixels.add(index + 2) = color.b(); // Blue
                    *self.pixels.add(index + 3) = 255;       // Alpha
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

/// Manejador de gráficos para FerroOS
pub struct GraphicsManager {
    framebuffer: FrameBuffer,
    current_color: Rgb888,
    cursor_x: i32,
    cursor_y: i32,
    line_height: i32,
}

impl GraphicsManager {
    pub fn new() -> Self {
        // Define the mailbox message buffer
        // The buffer must be 16-byte aligned
        #[repr(C, align(16))]
        struct MailboxBuffer {
            header: mailbox::MailboxMessage,
            // Set physical display size
            tag_set_physical_display_size: u32,
            value_buf_size_set_physical_display_size: u32,
            request_response_code_set_physical_display_size: u32,
            width: u32,
            height: u32,
            // Set virtual display size
            tag_set_virtual_display_size: u32,
            value_buf_size_set_virtual_display_size: u32,
            request_response_code_set_virtual_display_size: u32,
            vwidth: u32,
            vheight: u32,
            // Set depth
            tag_set_depth: u32,
            value_buf_size_set_depth: u32,
            request_response_code_set_depth: u32,
            depth: u32,
            // Allocate buffer
            tag_allocate_buffer: u32,
            value_buf_size_allocate_buffer: u32,
            request_response_code_allocate_buffer: u32,
            alignment: u32,
            // Response values for allocate buffer
            fb_address: u32,
            fb_size: u32,
            // Get pitch
            tag_get_pitch: u32,
            value_buf_size_get_pitch: u32,
            request_response_code_get_pitch: u32,
            pitch: u32,
            // End tag
            end_tag: u32,
        }

        let mut mbox_buffer = MailboxBuffer {
            header: mailbox::MailboxMessage {
                buffer_size: core::mem::size_of::<MailboxBuffer>() as u32,
                request_response_code: mailbox::MBOX_REQUEST,
            },
            // Set physical display size
            tag_set_physical_display_size: mailbox::MBOX_TAG_SET_PHYSICAL_DISPLAY_SIZE,
            value_buf_size_set_physical_display_size: 8,
            request_response_code_set_physical_display_size: 8,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            // Set virtual display size
            tag_set_virtual_display_size: mailbox::MBOX_TAG_SET_VIRTUAL_DISPLAY_SIZE,
            value_buf_size_set_virtual_display_size: 8,
            request_response_code_set_virtual_display_size: 8,
            vwidth: SCREEN_WIDTH,
            vheight: SCREEN_HEIGHT,
            // Set depth
            tag_set_depth: mailbox::MBOX_TAG_SET_DEPTH,
            value_buf_size_set_depth: 4,
            request_response_code_set_depth: 4,
            depth: 32, // 32 bits per pixel (RGBA8888)
            // Allocate buffer
            tag_allocate_buffer: mailbox::MBOX_TAG_ALLOCATE_BUFFER,
            value_buf_size_allocate_buffer: 8,
            request_response_code_allocate_buffer: 4, // Send 4 bytes (alignment)
            alignment: 4096, // 4096-byte alignment
            fb_address: 0,
            fb_size: 0,
            // Get pitch
            tag_get_pitch: mailbox::MBOX_TAG_GET_PITCH,
            value_buf_size_get_pitch: 4,
            request_response_code_get_pitch: 0,
            pitch: 0,
            // End tag
            end_tag: 0,
        };

        let mut framebuffer_address: u32 = 0;
        let mut pitch: u32 = 0;

        uart_send_str("  Enviando mensaje al mailbox...\n");
        // Send the mailbox message
        let result = mailbox::send_mailbox_message(&mut mbox_buffer.header);

        match result {
            Ok(_) => {
                uart_send_str("  ✅ Mensaje de mailbox enviado y procesado.\n");
                framebuffer_address = mbox_buffer.fb_address;
                pitch = mbox_buffer.pitch;
                
                // The GPU returns the address with the high bit set if it's a cached address.
                // We need to clear it to get the physical address.
                framebuffer_address &= 0x3FFF_FFFF;

                if framebuffer_address == 0 {
                    uart_send_str("  ⚠️ Framebuffer 0 detectado (QEMU VC4 falla allocation).\n");
                    uart_send_str("  🔧 Usando dirección fallback: 0x3C100000 (16-bit)\n");
                    framebuffer_address = 0x3C100000;
                    // Force pitch to 1280 (640 width * 2 bytes/pixel for RGB565)
                    pitch = 640 * 2;
                }

                uart_send_str("  Framebuffer Address: ");
                print_number(framebuffer_address as u64);
                uart_send_str("\n");
                uart_send_str("  Pitch: ");
                print_number(pitch as u64);
                uart_send_str("\n");
            }
            Err(_) => {
                uart_send_str("  ❌ Error al enviar mensaje al mailbox.\n");
                // Fallback to a known address if mailbox fails
                framebuffer_address = 0x3C100000; 
                pitch = 640 * 2;
            }
        }

        let mut manager = Self {
            framebuffer: FrameBuffer::new(framebuffer_address, pitch),
            current_color: colors::WHITE,
            cursor_x: 10,
            cursor_y: 30,
            line_height: 25,
        };
        
        // Inicializar con pantalla negra
        manager.clear_screen();
        manager
    }
    
    /// Limpiar pantalla
    pub fn clear_screen(&mut self) {
        self.framebuffer.clear(colors::BLACK);
        self.cursor_x = 10;
        self.cursor_y = 30;
    }
    
    /// Cambiar color actual
    pub fn set_color(&mut self, color: Rgb888) {
        self.current_color = color;
    }
    
    /// Fuente de 8x8 básica (ASCII 32-127)
    /// Para depuración sin dependencias externas
    const FONT_8X8: [u8; 768] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Space (32)
        0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00, // !
        0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, // "
        0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00, // #
        0x18, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x18, 0x00, // $
        0x00, 0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00, // %
        0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00, // &
        0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, // '
        0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00, // (
        0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00, // )
        0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00, // *
        0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00, // +
        0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30, // ,
        0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00, // -
        0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, // .
        0x00, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x00, 0x00, // /
        0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, // 0 (48)
        0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, // 1
        0x3C, 0x66, 0x06, 0x0C, 0x30, 0x60, 0x7E, 0x00, // 2
        0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00, // 3
        0x0C, 0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x00, // 4
        0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00, // 5
        0x3C, 0x60, 0xFC, 0x66, 0x66, 0x66, 0x3C, 0x00, // 6
        0x7E, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00, // 7
        0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00, // 8
        0x3C, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C, 0x00, // 9
        0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00, // :
        0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30, // ;
        0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00, // <
        0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00, // =
        0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00, // >
        0x3C, 0x66, 0x06, 0x0C, 0x18, 0x00, 0x18, 0x00, // ?
        0x3C, 0x66, 0x6E, 0x6E, 0x60, 0x62, 0x3C, 0x00, // @
        0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00, // A
        0xFC, 0x66, 0x66, 0x7C, 0x66, 0x66, 0xFC, 0x00, // B
        0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00, // C
        0xF8, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00, // D
        0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00, // E
        0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00, // F
        0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00, // G
        0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00, // H
        0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, // I
        0x1E, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00, // J
        0xE6, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0xE6, 0x00, // K
        0xF0, 0x60, 0x60, 0x60, 0x60, 0x60, 0xF0, 0x00, // L
        0xC6, 0xEE, 0xFE, 0xD6, 0xC6, 0xC6, 0xC6, 0x00, // M
        0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00, // N
        0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, // O
        0xFC, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00, // P
        0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x0E, 0x00, // Q
        0xFC, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0xE6, 0x00, // R
        0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00, // S
        0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, // T
        0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, // U
        0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00, // V
        0xC6, 0xC6, 0xC6, 0xD6, 0xFE, 0xEE, 0xC6, 0x00, // W
        0xC6, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0xC6, 0x00, // X
        0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00, // Y
        0xFE, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFE, 0x00, // Z
        0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00, // [
        0x00, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x00, 0x00, // Backslash
        0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00, // ]
        0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00, // ^
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, // _
        0x18, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, // `
        0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00, // a (97)
        0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00, // b
        0x00, 0x00, 0x3C, 0x60, 0x60, 0x60, 0x3C, 0x00, // c
        0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00, // d
        0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00, // e
        0x1C, 0x36, 0x30, 0x78, 0x30, 0x30, 0x30, 0x00, // f
        0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x3C, // g
        0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00, // h
        0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, // i
        0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C, // j
        0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00, // k
        0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, // l
        0x00, 0x00, 0xEC, 0xFE, 0xFE, 0xD6, 0xC6, 0x00, // m
        0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x00, // n
        0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00, // o
        0x00, 0x00, 0xDC, 0x66, 0x66, 0x7C, 0x60, 0xF0, // p
        0x00, 0x00, 0x76, 0x66, 0x66, 0x7C, 0x06, 0x1E, // q
        0x00, 0x00, 0xDC, 0x66, 0x60, 0x60, 0xF0, 0x00, // r
        0x00, 0x00, 0x3C, 0x60, 0x3C, 0x06, 0x7C, 0x00, // s
        0x30, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x1C, 0x00, // t
        0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, // u
        0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00, // v
        0x00, 0x00, 0xC6, 0xD6, 0xFE, 0xEE, 0xC4, 0x00, // w
        0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00, // x
        0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x0C, 0x78, // y
        0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00, // z
        0x0C, 0x18, 0x18, 0x30, 0x18, 0x18, 0x0C, 0x00, // {
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, // |
        0x30, 0x18, 0x18, 0x0C, 0x18, 0x18, 0x30, 0x00, // }
        0x36, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ~
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00  // DEL
    ];

    /// Dibujar un carácter manualmente
    pub fn draw_char_manual(&mut self, c: char, x: i32, y: i32) {
        let ascii_val = c as usize;
        let index = if ascii_val >= 32 && ascii_val <= 127 {
            ascii_val - 32
        } else {
            0
        };

        let bitmap = &Self::FONT_8X8[index * 8..(index + 1) * 8];
        let color = self.current_color;

        for (row_idx, row) in bitmap.iter().enumerate() {
            for col_idx in 0..8 {
                if (row >> (7 - col_idx)) & 1 == 1 {
                    let px = x + col_idx;
                    let py = y + row_idx as i32;
                    self.draw_pixel_manual(px, py, color);
                }
            }
        }
    }

    /// Dibujar un pixel manualmente
    fn draw_pixel_manual(&mut self, x: i32, y: i32, color: Rgb888) {
        if x < 0 || x >= SCREEN_WIDTH as i32 || y < 0 || y >= SCREEN_HEIGHT as i32 {
            return;
        }
        
        unsafe {
            if self.framebuffer.bytes_per_pixel == 2 {
                // RGB565 (16-bit)
                // RRRR RGGG GGGB BBBB
                let r5 = (color.r() >> 3) as u16;
                let g6 = (color.g() >> 2) as u16;
                let b5 = (color.b() >> 3) as u16;
                let rgb565 = (r5 << 11) | (g6 << 5) | b5;
                
                let offset = (y as u32 * self.framebuffer.pitch + x as u32 * 2) as usize;
                *(self.framebuffer.pixels.add(offset) as *mut u16) = rgb565;
            } else {
                // BGRA8888 (32-bit)
                let offset = (y as u32 * self.framebuffer.pitch + x as u32 * 4) as usize;
                *self.framebuffer.pixels.add(offset) = color.b();     // Blue
                *self.framebuffer.pixels.add(offset + 1) = color.g(); // Green
                *self.framebuffer.pixels.add(offset + 2) = color.r(); // Red
                *self.framebuffer.pixels.add(offset + 3) = 255;       // Alpha
            }
        }
    }

    /// Dibujar texto en la posición del cursor
    pub fn draw_text(&mut self, text: &str) {
        for c in text.chars() {
            self.draw_char_manual(c, self.cursor_x, self.cursor_y);
            self.cursor_x += 8;
        }
        
        // Avanzar cursor
        self.cursor_y += self.line_height;
        self.cursor_x = 10;
        
        // Verificar si necesitamos hacer scroll
        if self.cursor_y > (SCREEN_HEIGHT as i32 - 50) {
            self.scroll_up();
        }
    }
    
    /// Dibujar texto en posición específica
    pub fn draw_text_at(&mut self, text: &str, x: i32, y: i32) {
        let mut curr_x = x;
        for c in text.chars() {
            self.draw_char_manual(c, curr_x, y);
            curr_x += 8;
        }
    }
    
    /// Dibujar rectángulo
    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, filled: bool) {
        let rect = Rectangle::new(Point::new(x, y), Size::new(width, height));
        
        if filled {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(self.current_color)
                .build();
            rect.into_styled(style).draw(&mut self.framebuffer).ok();
        } else {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(self.current_color)
                .stroke_width(2)
                .build();
            rect.into_styled(style).draw(&mut self.framebuffer).ok();
        }
    }
    
    /// Nueva línea
    pub fn new_line(&mut self) {
        self.cursor_x = 10;
        self.cursor_y += self.line_height;
        
        if self.cursor_y > (SCREEN_HEIGHT as i32 - 50) {
            self.scroll_up();
        }
    }
    
    /// Scroll hacia arriba (simulado)
    fn scroll_up(&mut self) {
        // Simular scroll limpiando la pantalla y reiniciando
        // En una implementación real, moveríamos el contenido del buffer
        self.clear_screen();
    }
    
    /// Mostrar splash screen de FerroOS
    pub fn show_splash_screen(&mut self) {
        self.clear_screen();
        
        // Título principal
        self.set_color(colors::BLUE);
        self.draw_text_at("FerroOS Mobile", 280, 100);
        
        // Subtítulo
        self.set_color(colors::WHITE);
        self.draw_text_at("Sistema Operativo Nativo", 230, 140);
        
        // Versión
        self.set_color(colors::GREEN);
        self.draw_text_at("v1.0.0 - Zero Dependencies", 220, 180);
        
        // Pipeline
        self.set_color(colors::YELLOW);
        self.draw_text_at("Lua → Zig → WASM → Rust", 240, 220);
        
        // Borde decorativo
        self.set_color(colors::PURPLE);
        self.draw_rect(50, 50, SCREEN_WIDTH - 100, SCREEN_HEIGHT - 100, false);
        
        // Logo ASCII (simple)
        self.set_color(colors::ORANGE);
        self.draw_text_at("   ███████", 320, 280);
        self.draw_text_at("   █     █", 320, 300);
        self.draw_text_at("   █  ○  █", 320, 320);
        self.draw_text_at("   █     █", 320, 340);
        self.draw_text_at("   ███████", 320, 360);
        
        // Reset cursor para contenido normal
        self.cursor_x = 10;
        self.cursor_y = 420;
        self.set_color(colors::WHITE);
    }
}