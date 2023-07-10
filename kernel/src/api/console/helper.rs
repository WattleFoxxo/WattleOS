use core::{fmt, ptr};
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Colour {
    Clear = 0x00_00_00_00,
    Black = 0x00_00_00_FF,
    Blue = 0x00_00_AA_FF,
    Green = 0x00_AA_00_FF,
    Cyan = 0x00_AA_AA_FF,
    Red = 0xAA_00_00_FF,
    Magenta = 0xAA_00_AA_FF,
    Brown = 0xAA_55_00_FF,
    LightGray = 0xAA_AA_AA_FF,
    DarkGray = 0x55_55_55_FF,
    LightBlue = 0x55_55_FF_FF,
    LightGreen = 0x55_FF_55_FF,
    LightCyan = 0x55_FF_FF_FF,
    LightRed = 0xFF_55_55_FF,
    Pink = 0xFF_55_FF_FF,
    Yellow = 0xFF_FF_55_FF,
    White = 0xFF_FF_FF_FF,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub fg: u32,
    pub bg: u32,
}



pub struct TextBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<ScreenChar>,
}

impl TextBuffer {
    pub fn new(width: usize, height: usize) -> TextBuffer {
        let buffer_size = width * height;
        let default_char = ScreenChar {
            ascii_character: b' ',
            fg: 0xFFFFFFFF,
            bg: 0x00000000,
        };
        TextBuffer {
            width,
            height,
            buffer: vec![default_char; buffer_size],
        }
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<ScreenChar> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.buffer[index])
        } else {
            None
        }
    }

    pub fn set_char(&mut self, x: usize, y: usize, screen_char: ScreenChar) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index] = screen_char;
        }
    }

    pub fn get_row(&self, y: usize) -> Option<&[ScreenChar]> {
        if y < self.height {
            let start_index = y * self.width;
            let end_index = start_index + self.width;
            Some(&self.buffer[start_index..end_index])
        } else {
            None
        }
    }

    pub fn set_row(&mut self, y: usize, row: &[ScreenChar]) {
        if y < self.height && row.len() == self.width {
            let start_index = y * self.width;
            let end_index = start_index + self.width;
            self.buffer[start_index..end_index].copy_from_slice(row);
        }
    }
}


pub const LINE_SPACING: usize = 2;
pub const LETTER_SPACING: usize = 0;
pub const BORDER_PADDING: usize = 1;

pub mod font_constants {
    use super::*;
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
    pub const BACKUP_CHAR: char = '�';
    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

pub fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}
