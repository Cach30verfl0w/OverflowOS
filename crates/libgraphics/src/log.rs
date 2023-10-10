use crate::text::{
    set_color,
    write_char,
    write_str,
    DARK_BLUE,
    DARK_GRAY,
    GREEN,
    LIGHT_BLUE,
    ORANGE,
    RED,
    TEXT_WRITER_CONTEXT,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::RgbColor,
};
use log::{
    set_logger,
    set_max_level,
    Level,
    Log,
    Metadata,
    Record,
};

pub static LOGGER: GOPLogger = GOPLogger;

pub struct GOPLogger;

impl Log for GOPLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        unsafe { TEXT_WRITER_CONTEXT.is_some() }
    }

    fn log(&self, record: &Record) {
        set_color(Rgb888::BLACK, DARK_GRAY).unwrap();
        write_char('[').unwrap();
        match record.level() {
            Level::Error => {
                set_color(Rgb888::BLACK, RED).unwrap();
                write_str("Error").unwrap();
            }
            Level::Warn => {
                set_color(Rgb888::BLACK, ORANGE).unwrap();
                write_str("Warn").unwrap();
            }
            Level::Info => {
                set_color(Rgb888::BLACK, GREEN).unwrap();
                write_str("Info").unwrap();
            }
            Level::Debug => {
                set_color(Rgb888::BLACK, LIGHT_BLUE).unwrap();
                write_str("Debug").unwrap();
            }
            Level::Trace => {
                set_color(Rgb888::BLACK, DARK_BLUE).unwrap();
                write_str("Trace").unwrap();
            }
        }
        set_color(Rgb888::BLACK, DARK_GRAY).unwrap();
        write_char(']').unwrap();

        set_color(Rgb888::BLACK, Rgb888::WHITE).unwrap();
        write_char(' ').unwrap();
        unsafe { TEXT_WRITER_CONTEXT.as_mut().unwrap() }
            .write_fmt(record.args().clone())
            .unwrap();
        crate::swap_buffers().unwrap();
    }

    fn flush(&self) {}
}

pub fn install_logger() -> Result<(), log::SetLoggerError> {
    set_max_level(log::STATIC_MAX_LEVEL);
    set_logger(&LOGGER)
}
