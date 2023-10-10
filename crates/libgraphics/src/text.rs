use crate::{
    embedded_graphics::Drawable,
    error::Error,
    GRAPHICS_CONTEXT,
};
use core::fmt;
use embedded_graphics::{
    mono_font::{
        MonoFont,
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::{
        Point,
        RgbColor,
    },
    text::{
        Alignment,
        Text,
        TextStyleBuilder,
    },
};

pub static DARK_GRAY: Rgb888 = Rgb888::new(90, 90, 90);
pub static RED: Rgb888 = Rgb888::new(255, 0, 0);
pub static GREEN: Rgb888 = Rgb888::new(0, 255, 0);
pub static ORANGE: Rgb888 = Rgb888::new(153, 76, 0);
pub static DARK_BLUE: Rgb888 = Rgb888::new(0, 0, 204);
pub static LIGHT_BLUE: Rgb888 = Rgb888::new(51, 51, 255);

pub static mut TEXT_WRITER_CONTEXT: Option<TextWriterContext> = None;

pub struct TextWriterContext<'a> {
    font: MonoFont<'a>,
    current_x: usize,
    current_y: usize,
    current_foreground_color: Rgb888,
    current_background_color: Rgb888,
}

impl fmt::Write for TextWriterContext<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s).unwrap();
        Ok(())
    }
}

pub fn create_text_writer_context(font: MonoFont<'static>) -> Result<(), Error> {
    if unsafe { TEXT_WRITER_CONTEXT.is_some() } {
        return Err(Error::ContextAlreadyCreated);
    }

    if unsafe { GRAPHICS_CONTEXT.is_none() } {
        return Err(Error::NoContext);
    }

    unsafe {
        TEXT_WRITER_CONTEXT = Some(TextWriterContext {
            font,
            current_x: 0,
            current_y: 0,
            current_foreground_color: Rgb888::WHITE,
            current_background_color: Rgb888::BLACK,
        });
    }
    Ok(())
}

pub fn invalidate_text_write_context() -> Result<(), Error> {
    if unsafe { TEXT_WRITER_CONTEXT.is_none() } {
        return Err(Error::NoContext);
    }

    unsafe { TEXT_WRITER_CONTEXT = None };
    Ok(())
}

pub fn write_char(char: char) -> Result<(), Error> {
    let graphics_context = unsafe { GRAPHICS_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    let text_writer_context =
        unsafe { TEXT_WRITER_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;

    let mut buffer = [0u8; 2];
    Text::with_text_style(
        char.encode_utf8(&mut buffer),
        Point::new(
            (text_writer_context.current_x * text_writer_context.font.character_size.width as usize)
                as i32,
            (text_writer_context.current_y * text_writer_context.font.character_size.height as usize)
                as i32,
        ),
        MonoTextStyleBuilder::new()
            .font(&text_writer_context.font)
            .text_color(text_writer_context.current_foreground_color)
            .background_color(text_writer_context.current_background_color)
            .build(),
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(embedded_graphics::text::Baseline::Top)
            .build(),
    )
    .draw(graphics_context)?;

    text_writer_context.current_x += 1;
    if text_writer_context.current_x >= graphics_context.current_mode.stride() {
        next_row()?;
    }
    Ok(())
}

pub fn write_str(string: &str) -> Result<(), Error> {
    for char in string.chars() {
        match char {
            '\n' => next_row()?,
            _ => write_char(char)?,
        }
    }
    Ok(())
}

pub fn set_color(background_color: Rgb888, foreground_color: Rgb888) -> Result<(), Error> {
    let context = unsafe { TEXT_WRITER_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    context.current_foreground_color = foreground_color;
    context.current_background_color = background_color;
    Ok(())
}

pub fn next_row() -> Result<(), Error> {
    let context = unsafe { TEXT_WRITER_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    context.current_y += 1;
    context.current_x = 0;
    Ok(())
}
