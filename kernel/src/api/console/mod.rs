use core::{fmt, ptr};
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

use crate::io::vga;

mod helper;
pub use helper::*;

pub mod palette;
pub use palette::Palette;

use crate::io::vga::font;

const BUFFER_WIDTH: usize = 142; //80;
const BUFFER_HEIGHT: usize = 62; //25;

pub static COLOUR_PALETTE: Palette = palette::Flat; //25;

//pub static FONT_CONFIG: font::Config = font::cozette::CONFIG;

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        col: 0,
        row: 0,
        width: BUFFER_WIDTH,
        height: BUFFER_HEIGHT,
        buffer: TextBuffer::new(BUFFER_WIDTH, BUFFER_HEIGHT),
    });
}

pub struct Console {
    col: usize,
    row: usize,
    width: usize,
    height: usize,
    buffer: TextBuffer,
}

impl Console {
    fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row >= BUFFER_HEIGHT-1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.get_char(col, row).expect("uh oh");
                    self.buffer.set_char(col, row - 1, character);
                }
            }
            self.row -= 1;
            vga::shift_y(13);
        }
        
    }

    fn clear_row(&mut self, row: usize) {
        let y_pos = row * font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
        let blank = ScreenChar {
            ascii_character: b' ',
            fg: 0xFF_FF_FF_FF,
            bg: 0x00_00_00_FF
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.set_char(col, row, blank);
        }
    }

    fn clear(&mut self) {
        self.col = 0;
        self.row = 0;
        vga::clear(0x00_00_00_FF);
    }

    fn write_byte(&mut self, byte: u8, fg_colour: u32, bg_colour: u32) {
        match byte {
            b'\n' => self.new_line(),

            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                let screen_char = ScreenChar {
                    ascii_character: byte,
                    fg: fg_colour,
                    bg: bg_colour,
                };

                self.buffer.set_char(self.col, self.row, screen_char);
                self.write_char(self.col, self.row, screen_char);
                self.col += 1;
            }
        }
    }

    fn write_char(&mut self, x: usize, y: usize, screen_char: ScreenChar) {
        let width = font_constants::CHAR_RASTER_WIDTH;
        let height = font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
        let x_pos = x * 7;
        let y_pos = y * 13;
        vga::char_bitmap(x_pos, y_pos, 1, screen_char.fg, screen_char.bg, screen_char.ascii_character as char);
    }
}

unsafe impl Send for Console {}
unsafe impl Sync for Console {}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let colour_palette = &COLOUR_PALETTE;
        let mut mode = 0;
        let mut colourmode = true;
        let mut fg = colour_palette.white;
        let mut bg = colour_palette.clear;
        for c in s.chars() {
            match mode {
                0 => match c {
                    '\x1b' => mode = 1,
                    _ => self.write_byte(c as u8, fg, bg),
                },
                1 => match c {
                    ';' => mode = 2,
                    'm' => mode = 0,
                    '0' => {
                        fg = colour_palette.white;
                        bg = colour_palette.clear;
                    },
                    '1' => colourmode = false,
                    '2' => colourmode = true,
                    _ => {
                        mode = 0;
                        self.write_byte(c as u8, fg, bg);
                    },
                },
                2 => match c {
                    'm' => mode = 0,
                    'r' => match colourmode { true => { fg = colour_palette.clear; }, false => { bg = colour_palette.clear; } },
                    '0' => match colourmode { true => { fg = colour_palette.black; }, false => { bg = colour_palette.black; } },
                    '1' => match colourmode { true => { fg = colour_palette.blue; }, false => { bg = colour_palette.blue; } },
                    '2' => match colourmode { true => { fg = colour_palette.green; }, false => { bg = colour_palette.green; } },
                    '3' => match colourmode { true => { fg = colour_palette.cyan; }, false => { bg = colour_palette.cyan; } },
                    '4' => match colourmode { true => { fg = colour_palette.red; }, false => { bg = colour_palette.red; } },
                    '5' => match colourmode { true => { fg = colour_palette.magenta; }, false => { bg = colour_palette.magenta; } },
                    '6' => match colourmode { true => { fg = colour_palette.brown; }, false => { bg = colour_palette.brown; } },
                    '7' => match colourmode { true => { fg = colour_palette.lightgray; }, false => { bg = colour_palette.lightgray; } },
                    '8' => match colourmode { true => { fg = colour_palette.darkgray; }, false => { bg = colour_palette.darkgray; } },
                    '9' => match colourmode { true => { fg = colour_palette.lightblue; }, false => { bg = colour_palette.lightblue; } },
                    'a' => match colourmode { true => { fg = colour_palette.lightgreen; }, false => { bg = colour_palette.lightgreen; } },
                    'b' => match colourmode { true => { fg = colour_palette.lightcyan; }, false => { bg = colour_palette.lightcyan; } },
                    'c' => match colourmode { true => { fg = colour_palette.lightred; }, false => { bg = colour_palette.lightred; } },
                    'd' => match colourmode { true => { fg = colour_palette.pink; }, false => { bg = colour_palette.pink; } },
                    'e' => match colourmode { true => { fg = colour_palette.yellow; }, false => { bg = colour_palette.yellow; } },
                    'f' => match colourmode { true => { fg = colour_palette.white; }, false => { bg = colour_palette.white; } },

                    _ => {
                        mode = 0;
                        self.write_byte(c as u8, fg, bg);
                    },
                }
                
                _ => {},
            }
        }
        vga::flip();
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        CONSOLE.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::api::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

fn parse_color_string(color_string: &str) -> Option<(u8, u8, u8)> {
    if let Some(start_index) = color_string.find("[38;2;") {
        let end_index = color_string.find("m").unwrap_or(color_string.len());
        let color_values = &color_string[start_index + 7..end_index];

        let values: Vec<u8> = color_values
            .split(';')
            .filter_map(|value| value.parse().ok())
            .collect();

        if values.len() == 3 {
            return Some((values[0], values[1], values[2]));
        }
    }

    None
}