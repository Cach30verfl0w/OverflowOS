use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyleBuilder};
use embedded_graphics::{Drawable, Pixel};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Point, RgbColor};
use embedded_graphics::text::{Alignment, Text, TextStyleBuilder};
use uefi::Identify;
use uefi::prelude::BootServices;
use uefi::proto::console::gop::{GraphicsOutput, ModeInfo};
use uefi::table::boot::{MemoryType, ScopedProtocol, SearchType};
use crate::error::Error;

pub struct UEFIFramebuffer<'a> {
    protocol: ScopedProtocol<'a, GraphicsOutput>,
    current_mode: ModeInfo,
    framebuffer: &'a mut [u32],
    swap_buffer: &'a mut [u32]
}

impl OriginDimensions for UEFIFramebuffer<'_> {
    fn size(&self) -> Size {
        Size::new(
            self.current_mode.resolution().0 as u32,
            self.current_mode.resolution().1 as u32
        )
    }
}

impl DrawTarget for UEFIFramebuffer<'_> {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        for Pixel(point, color) in pixels {
            self.set_pixel_at(point.x as usize, point.y as usize, color);
        }
        Ok(())
    }
}

impl<'a> UEFIFramebuffer<'a> {
    pub fn new(boot_services: &'a BootServices) -> Result<Self, Error> {
        let first_handle = *boot_services
            .locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
            .unwrap().first().unwrap();
        let mut protocol: ScopedProtocol<'a, GraphicsOutput> = boot_services
            .open_protocol_exclusive(first_handle)?;

        let memory = boot_services.allocate_pool(MemoryType::LOADER_DATA,
                                                 protocol.frame_buffer().size()).unwrap();

        Ok(Self {
            framebuffer: unsafe {
                core::slice::from_raw_parts_mut(
                    protocol.frame_buffer().as_mut_ptr() as *mut u32,
                    protocol.frame_buffer().size()
                )
            },
            current_mode: protocol.current_mode_info(),
            swap_buffer: unsafe {
                core::slice::from_raw_parts_mut(
                    memory as *mut u32,
                    protocol.frame_buffer().size()
                )
            },
            protocol,
        })
    }

    pub fn fill(&mut self, start_x: usize, start_y: usize, width: usize, height: usize, color: Rgb888) {
        for x in 0..width {
            for y in 0..height {
                self.set_pixel_at(start_x + x, start_y + y, color);
            }
        }
    }

    #[inline]
    pub fn set_pixel_at(&mut self, x: usize, y: usize, color: Rgb888) {
        *self.swap_buffer.get_mut(y * self.current_mode.stride() + x).unwrap() = (
            (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32)
        );
    }

    #[inline]
    pub fn get_pixel_at(&self, x: usize, y: usize) -> u32 {
        *self.swap_buffer.get(y * self.current_mode.stride() + x).unwrap()
    }

    pub fn clear(&mut self) {
        for y in 0..self.current_mode.resolution().1 {
            for x in 0..self.current_mode.stride() {
                self.set_pixel_at(x, y, Rgb888::BLACK);
            }
        }
    }

    pub fn swap_buffer(&mut self) {
        self.framebuffer.copy_from_slice(self.swap_buffer);
    }

}

pub struct TextWriter<'a, 'b> {
    framebuffer: &'a mut UEFIFramebuffer<'b>,
    font: MonoFont<'a>,
    current_x: usize,
    current_y: usize
}

impl<'a, 'b> TextWriter<'a, 'b> {
    pub fn new(value: &'a mut UEFIFramebuffer<'b>, font: MonoFont<'a>) -> Self {
        Self {
            framebuffer: value,
            current_x: 0,
            current_y: 0,
            font
        }
    }
}

impl TextWriter<'_, '_> {
    pub fn write_char(&mut self, char: char) {
        // TODO: Remove clone
        let font = self.font.clone();

        let mut buffer = [0u8; 2];
        Text::with_text_style(
            char.encode_utf8(&mut buffer),
            Point::new((self.current_x * self.font.character_size.width as usize) as i32, (self.current_y * self.font.character_size.height as usize) as i32),
            MonoTextStyleBuilder::new()
                .font(&font)
                .text_color(Rgb888::WHITE)
                .background_color(Rgb888::BLACK)
                .build(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(embedded_graphics::text::Baseline::Top)
                .build()
        ).draw(self.framebuffer).unwrap();

        self.current_x += 1;
        if self.current_x >= self.framebuffer.current_mode.resolution().1 / self.font.character_size.height as usize {
            self.next_row();
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for char in string.chars() {
            match char {
                '\n' => self.next_row(),
                _ => self.write_char(char)
            }
        }
        self.framebuffer.swap_buffer();
    }

    fn next_row(&mut self) {
        self.current_y += 1;
        self.current_x = 0;
    }
}