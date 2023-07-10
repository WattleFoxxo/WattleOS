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

pub static FONT_CONFIG: font::Config = font::cozette::CONFIG;

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        col: 0,
        row: 0,
        buffer: TextBuffer::new(1, 1),
        colour_palette: palette::Flat,
    });
}

pub struct Console {
    col: usize,
    row: usize,
    buffer: TextBuffer,
    colour_palette: Palette,
}

impl Console {

    pub fn init(&mut self, palette: Palette) {
        self.colour_palette = palette;
        let width = vga::width()/FONT_CONFIG.width;
        let height = (vga::height()/FONT_CONFIG.height)+1;

        self.buffer = TextBuffer::new(width, height);
    }

    fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row >= self.buffer.height-1 {
            for row in 1..self.buffer.height {
                for col in 0..self.buffer.width {
                    let character = self.buffer.get_char(col, row).expect("uh oh");
                    self.buffer.set_char(col, row - 1, character);
                }
            }
            self.row -= 1;
            vga::shift_y(13);
            vga::rect(0, 0, vga::width(), 1, self.colour_palette.black); //TODO: fix jank
            vga::rect(0, vga::height()-13, vga::width(), FONT_CONFIG.height+5, self.colour_palette.black);
        }
        
    }

    pub fn back_space(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            let default_char = ScreenChar {
                ascii_character: b' ',
                fg: self.colour_palette.black,
                bg: self.colour_palette.black,
            };
            self.write_char(self.col, self.row, default_char);
            //self.col -= 1;
            vga::flip();
        }
        
    }

    fn fill(&mut self, c: char, fg_colour: u32, bg_colour: u32) {
        self.col = 0;
        self.row = 0;

        let default_char = ScreenChar {
            ascii_character: b' ',
            fg: fg_colour,
            bg: bg_colour,
        };
        
        for row in 1..self.buffer.height {
            for col in 0..self.buffer.width {
                if c != ' ' {
                    self.write_byte(c as u8, fg_colour, bg_colour);
                } else {
                    self.buffer.set_char(col, row, default_char);
                }
            }
        }

        vga::clear(bg_colour);
    }

    fn write_byte(&mut self, byte: u8, fg_colour: u32, bg_colour: u32) {
        match byte {
            b'\n' => self.new_line(),

            byte => {
                if self.col >= self.buffer.width {
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

        // self.write_char(0, self.buffer.height-2, ScreenChar {
        //     ascii_character: b'#',
        //     fg: self.colour_palette.white,
        //     bg: self.colour_palette.white,
        // });

        // self.write_char(self.buffer.width-1, self.buffer.height-2, ScreenChar {
        //     ascii_character: b'#',
        //     fg: self.colour_palette.white,
        //     bg: self.colour_palette.white,
        // });

        // self.write_char(self.buffer.width-1, 0, ScreenChar {
        //     ascii_character: b'#',
        //     fg: self.colour_palette.white,
        //     bg: self.colour_palette.white,
        // });
    }

    fn write_char(&mut self, x: usize, y: usize, screen_char: ScreenChar) {
        let x_pos = x * FONT_CONFIG.width;
        let y_pos = y * FONT_CONFIG.height;
        vga::char_bitmap(x_pos, y_pos, 1, screen_char.fg, screen_char.bg, screen_char.ascii_character as char);
    }

    pub fn get_palette(&mut self) -> Palette {
        self.colour_palette
    }
}

unsafe impl Send for Console {}
unsafe impl Sync for Console {}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut mode = 0;
        let mut colourmode = true;
        let mut fg = self.colour_palette.white;
        let mut bg = self.colour_palette.black;
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
                        fg = self.colour_palette.white;
                        bg = self.colour_palette.black;
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
                    'r' => match colourmode { true => { fg = self.colour_palette.clear; }, false => { bg = self.colour_palette.clear; } },
                    '0' => match colourmode { true => { fg = self.colour_palette.black; }, false => { bg = self.colour_palette.black; } },
                    '1' => match colourmode { true => { fg = self.colour_palette.blue; }, false => { bg = self.colour_palette.blue; } },
                    '2' => match colourmode { true => { fg = self.colour_palette.green; }, false => { bg = self.colour_palette.green; } },
                    '3' => match colourmode { true => { fg = self.colour_palette.cyan; }, false => { bg = self.colour_palette.cyan; } },
                    '4' => match colourmode { true => { fg = self.colour_palette.red; }, false => { bg = self.colour_palette.red; } },
                    '5' => match colourmode { true => { fg = self.colour_palette.magenta; }, false => { bg = self.colour_palette.magenta; } },
                    '6' => match colourmode { true => { fg = self.colour_palette.brown; }, false => { bg = self.colour_palette.brown; } },
                    '7' => match colourmode { true => { fg = self.colour_palette.lightgray; }, false => { bg = self.colour_palette.lightgray; } },
                    '8' => match colourmode { true => { fg = self.colour_palette.darkgray; }, false => { bg = self.colour_palette.darkgray; } },
                    '9' => match colourmode { true => { fg = self.colour_palette.lightblue; }, false => { bg = self.colour_palette.lightblue; } },
                    'a' => match colourmode { true => { fg = self.colour_palette.lightgreen; }, false => { bg = self.colour_palette.lightgreen; } },
                    'b' => match colourmode { true => { fg = self.colour_palette.lightcyan; }, false => { bg = self.colour_palette.lightcyan; } },
                    'c' => match colourmode { true => { fg = self.colour_palette.lightred; }, false => { bg = self.colour_palette.lightred; } },
                    'd' => match colourmode { true => { fg = self.colour_palette.pink; }, false => { bg = self.colour_palette.pink; } },
                    'e' => match colourmode { true => { fg = self.colour_palette.yellow; }, false => { bg = self.colour_palette.yellow; } },
                    'f' => match colourmode { true => { fg = self.colour_palette.white; }, false => { bg = self.colour_palette.white; } },

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

pub fn init(palette: Palette) {
    CONSOLE.lock().init(palette);
}

pub fn clear() {
    fill(' ', palette().black, palette().black);
}

pub fn fill(c: char, fg_colour: u32, bg_colour: u32) {
    CONSOLE.lock().fill(c, fg_colour, bg_colour);
    vga::flip();
}

pub fn back_space() {
    CONSOLE.lock().back_space();
}

pub fn set_position(row: usize, col: usize) {
    //CONSOLE.lock().set_position(row, col);
}

pub fn palette() -> Palette {
    CONSOLE.lock().get_palette()
}
