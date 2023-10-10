use crate::{
    embedded_graphics::Drawable,
    error::Error,
    GRAPHICS_CONTEXT,
};
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

static mut TEXT_WRITER_CONTEXT: Option<TextWriterContext> = None;

struct TextWriterContext<'a> {
    font: MonoFont<'a>,
    current_x: usize,
    current_y: usize,
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
            .text_color(Rgb888::WHITE)
            .background_color(Rgb888::BLACK)
            .build(),
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(embedded_graphics::text::Baseline::Top)
            .build(),
    )
    .draw(graphics_context)?;

    text_writer_context.current_x += 1;
    if text_writer_context.current_x
        >= graphics_context.current_mode.resolution().1
            / text_writer_context.font.character_size.height as usize
    {
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

pub fn next_row() -> Result<(), Error> {
    let context = unsafe { TEXT_WRITER_CONTEXT.as_mut() }.ok_or_else(|| Error::NoContext)?;
    context.current_y += 1;
    context.current_x = 0;
    Ok(())
}
