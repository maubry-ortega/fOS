//! FerroOS Mobile - Módulo de gráficos embebido
//! Sistema de gráficos básico para mostrar UI en pantalla

use embedded_graphics::{mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder}, pixelcolor::Rgb888, prelude::*, primitives::{Rectangle, PrimitiveStyleBuilder}, text::{Baseline, Text}};

use crate::mailbox;
use crate::uart_send_str;
use fos_microkernel::print_number;

/// Resolución de pantalla por defecto para dispositivos móviles
pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;

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
        Self {
            pixels: framebuffer_address as *mut u8,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            bytes_per_pixel: 4, // RGBA8888
            pitch,
        }
    }
    
    /// Limpiar pantalla con un color
    pub fn clear(&mut self, color: Rgb888) {
        let (r, g, b) = (color.r(), color.g(), color.b());
        
        unsafe {
            for y in 0..self.height {
                for x in 0..self.width {
                    let offset = (y * self.pitch + x * self.bytes_per_pixel) as usize;
                    *self.pixels.add(offset) = r;     // Red
                    *self.pixels.add(offset + 1) = g; // Green
                    *self.pixels.add(offset + 2) = b; // Blue
                    *self.pixels.add(offset + 3) = 255; // Alpha
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
            request_response_code_set_physical_display_size: 0,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            // Set virtual display size
            tag_set_virtual_display_size: mailbox::MBOX_TAG_SET_VIRTUAL_DISPLAY_SIZE,
            value_buf_size_set_virtual_display_size: 8,
            request_response_code_set_virtual_display_size: 0,
            vwidth: SCREEN_WIDTH,
            vheight: SCREEN_HEIGHT,
            // Set depth
            tag_set_depth: mailbox::MBOX_TAG_SET_DEPTH,
            value_buf_size_set_depth: 4,
            request_response_code_set_depth: 0,
            depth: 32, // 32 bits per pixel (RGBA8888)
            // Allocate buffer
            tag_allocate_buffer: mailbox::MBOX_TAG_ALLOCATE_BUFFER,
            value_buf_size_allocate_buffer: 8,
            request_response_code_allocate_buffer: 0,
            alignment: 16, // 16-byte alignment
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
                uart_send_str("  Framebuffer Address: ");
                print_number(framebuffer_address as u64);
                uart_send_str("\n");
                uart_send_str("  Pitch: ");
                print_number(pitch as u64);
                uart_send_str("\n");

                // The GPU returns the address with the high bit set if it's a cached address.
                // We need to clear it to get the physical address.
                framebuffer_address &= 0x3FFF_FFFF;
            }
            Err(_) => {
                uart_send_str("  ❌ Error al enviar mensaje al mailbox.\n");
                // Fallback to a known address if mailbox fails (though this won't work for raspi4b)
                framebuffer_address = 0x4000_0000; 
                pitch = SCREEN_WIDTH * 4; // Default pitch
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
    
    /// Dibujar texto en la posición del cursor
    pub fn draw_text(&mut self, text: &str) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_9X18_BOLD)
            .text_color(self.current_color)
            .build();
        
        Text::with_baseline(
            text,
            Point::new(self.cursor_x, self.cursor_y),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.framebuffer)
        .ok();
        
        // Avanzar cursor
        self.cursor_y += self.line_height;
        
        // Verificar si necesitamos hacer scroll
        if self.cursor_y > (SCREEN_HEIGHT as i32 - 50) {
            self.scroll_up();
        }
    }
    
    /// Dibujar texto en posición específica
    pub fn draw_text_at(&mut self, text: &str, x: i32, y: i32) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_9X18_BOLD)
            .text_color(self.current_color)
            .build();
        
        Text::with_baseline(
            text,
            Point::new(x, y),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.framebuffer)
        .ok();
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