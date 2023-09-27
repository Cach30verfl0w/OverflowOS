use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Write;
use log::{Level, Log, Metadata, Record};
use spin::Mutex;
use uefi::CString16;
use uefi::proto::console::text::Color;
use crate::SYSTEM_TABLE;

/// This structure provides a UEFI-based color facade
pub(crate) struct ColorStack {
    color_stack: Vec<(Color, Color)>,
    current_color: (Color, Color)
}

impl ColorStack {

    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            color_stack: Vec::new(),
            current_color: (Color::White, Color::Black)
        }
    }

    #[inline]
    pub(crate) fn push_color(&mut self, foreground: Color, background: Color) {
        self.set_color(foreground, background);
        self.color_stack.push((foreground, background));
    }

    pub(crate) fn pop_color(&mut self) {
        self.color_stack.remove(self.color_stack.len() - 1);
        if self.color_stack.len() == 0 {
            self.set_color(Color::White, Color::Black);
            return;
        }

        let color = self.color_stack.get(self.color_stack.len() - 1).unwrap();
        self.set_color(color.0, color.1);
    }

    #[inline]
    pub(crate) fn set_color(&mut self, foreground: Color, background: Color) {
        self.current_color = (foreground, background);
        unsafe { SYSTEM_TABLE.as_mut() }.unwrap().stdout().set_color(foreground, background).unwrap();
    }

}

/// This structure is the implementation of a logger into the log create
pub(crate) struct Logger {
    max_level: Level,
    color_stack: Mutex<RefCell<ColorStack>>
}

impl Logger {

    pub(crate) fn new(max_level: Level) -> Self {
        Self {
            max_level,
            color_stack: Mutex::new(RefCell::new(ColorStack::new()))
        }
    }

}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        &metadata.level().to_level_filter() <= &self.max_level
    }

    fn log(&self, record: &Record) {
        let color_stack_guard = self.color_stack.lock();
        let mut color_stack = color_stack_guard.borrow_mut();

        // Print level with colors
        color_stack.push_color(Color::DarkGray, Color::Black);
        Logger::output_string("[");
        match record.level() {
            Level::Error => {
                color_stack.push_color(Color::Red, Color::Black);
                Logger::output_string(" Error ");
            }
            Level::Warn => {
                color_stack.push_color(Color::Yellow, Color::Black);
                Logger::output_string(" Warn  ");
            }
            Level::Info => {
                color_stack.push_color(Color::Green, Color::Black);
                Logger::output_string(" Info  ");
            }
            Level::Debug => {
                color_stack.push_color(Color::Cyan, Color::Black);
                Logger::output_string(" Debug ");
            }
            Level::Trace => {
                color_stack.push_color(Color::LightCyan, Color::Black);
                Logger::output_string(" Trace ");
            }
        }
        color_stack.pop_color();
        Logger::output_string("] ");
        color_stack.pop_color();

        // Print log message
        unsafe { SYSTEM_TABLE.as_mut() }.unwrap().stdout().write_fmt(record.args().clone()).unwrap();
    }

    fn flush(&self) {}
}

impl Logger {

    #[inline]
    pub(crate) fn output_string(message: &str) {
        unsafe { SYSTEM_TABLE.as_mut() }.unwrap().stdout().output_string(CString16::try_from(message)
            .unwrap().as_ref()).unwrap();
    }

}