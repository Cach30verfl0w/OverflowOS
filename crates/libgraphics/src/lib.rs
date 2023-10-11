#![no_std]

extern crate alloc;

pub mod error;
pub mod log;
pub mod text;

use crate::error::Error;
use ::embedded_graphics::image::Image;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};
use uefi::{
    prelude::BootServices,
    proto::console::gop::{
        GraphicsOutput,
        ModeInfo,
    },
    table::boot::{
        MemoryType,
        ScopedProtocol,
        SearchType,
    },
    Identify,
};

pub mod embedded_graphics {
    pub use embedded_graphics::*;
}

pub static mut GRAPHICS_CONTEXT: Option<GraphicsContext> = None;

pub struct GraphicsContext<'a> {
    swap_buffer: &'a mut [u32],
    framebuffer: &'a mut [u32],
    current_mode: ModeInfo,
}

impl OriginDimensions for GraphicsContext<'_> {
    fn size(&self) -> Size {
        Size::new(self.current_mode.resolution().0 as u32, self.current_mode.resolution().1 as u32)
    }
}

impl DrawTarget for GraphicsContext<'_> {
    type Color = Rgb888;
    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            set_pixel_at(point.x as usize, point.y as usize, color)?;
        }
        Ok(())
    }
}

/// This function tries to get the GraphicsOutputProtocol (GOP) and creates a Graphics Context for
/// all graphical operations with the help of GOP. The context must be created, when the UEFI
/// application is in the Boot Services, so this library can allocate the memory for a swap buffer.
pub fn create_context<'a>(boot_services: &'a BootServices) -> Result<(), Error> {
    if unsafe { GRAPHICS_CONTEXT.is_some() } {
        return Err(Error::ContextAlreadyCreated);
    }

    let first_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
        .unwrap()
        .first()
        .unwrap();
    let mut protocol: ScopedProtocol<'a, GraphicsOutput> =
        boot_services.open_protocol_exclusive(first_handle)?;

    let memory = boot_services
        .allocate_pool(MemoryType::LOADER_DATA, protocol.frame_buffer().size())
        .unwrap();

    unsafe {
        GRAPHICS_CONTEXT = Some(GraphicsContext {
            framebuffer: core::slice::from_raw_parts_mut(
                protocol.frame_buffer().as_mut_ptr() as *mut u32,
                protocol.frame_buffer().size(),
            ),
            current_mode: protocol.current_mode_info(),
            swap_buffer: core::slice::from_raw_parts_mut(
                memory as *mut u32,
                protocol.frame_buffer().size(),
            ),
        });
    }
    Ok(())
}

/// This function sets the specified color on the specified positions, if the context was already
/// created. If no context is created, this function returns a [Error::NoContext] error.
pub fn set_pixel_at(x: usize, y: usize, color: Rgb888) -> Result<(), Error> {
    let context = unsafe { GRAPHICS_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    *context
        .swap_buffer
        .get_mut(y * context.current_mode.stride() + x)
        .ok_or_else(|| Error::OutOfBounds)? =
        (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32);
    Ok(())
}

/// This function gets the color on the specified positions, if the context was already created. If
/// no context is created, this function returns a [Error::NoContext] error.
pub fn get_pixel_at(x: usize, y: usize) -> Result<u32, Error> {
    let context = unsafe { GRAPHICS_CONTEXT.as_ref() }.ok_or_else(|| Error::NoContext)?;
    Ok(*context
        .framebuffer
        .get(y * context.current_mode.stride() + x)
        .ok_or_else(|| Error::OutOfBounds)?)
}

/// This function fills the complete buffer with the specified color, if the context was already
/// created. If no context is created, this function returns a [Error::NoContext] error.
pub fn fill_buffer(color: Rgb888) -> Result<(), Error> {
    let (width, height) = unsafe { GRAPHICS_CONTEXT.as_ref() }
        .ok_or_else(|| Error::NoContext)?
        .current_mode
        .resolution();

    for x in 0..width {
        for y in 0..height {
            set_pixel_at(x, y, color)?;
        }
    }
    Ok(())
}

/// This function fills the specified region of the framebuffer with the specified color. If no
/// context is created, this function returns a [Error::NoContext] error.
pub fn fill(x: usize, y: usize, width: usize, height: usize, color: Rgb888) -> Result<(), Error> {
    for cx in x..(x + width) {
        for cy in y..(y + height) {
            set_pixel_at(cx, cy, color)?;
        }
    }
    Ok(())
}

/// This functions creates a image at the specified position and writes it into the framebuffer. If
/// no context is created, this function returns a [Error::NoContext] error.
pub fn draw_image<T: ImageDrawable<Color = Rgb888>>(
    image: &T, x: usize, y: usize,
) -> Result<(), Error> {
    let context = unsafe { GRAPHICS_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    let image = Image::new(image, Point::new(x as i32, y as i32));
    image.draw(context)?;
    Ok(())
}

/// This function copies the content of the swap buffer into the frame buffer and shows the drawn
/// screen to the user. If no context is created, this function returns a [Error::NoContext] error.
pub fn swap_buffers() -> Result<(), Error> {
    let context = unsafe { GRAPHICS_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    context.framebuffer.copy_from_slice(context.swap_buffer);
    Ok(())
}

pub fn resolution() -> Result<(usize, usize), Error> {
    Ok(unsafe { GRAPHICS_CONTEXT.as_mut() }
        .ok_or_else(|| Error::NoContext)?
        .current_mode
        .resolution())
}
