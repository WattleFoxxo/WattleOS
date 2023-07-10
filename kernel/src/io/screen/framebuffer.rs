use core::{fmt, ptr, slice};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::alloc::Layout;
use noto_sans_mono_bitmap::RasterizedChar;
use core::cmp::max;

use bootloader_api::info::{FrameBufferInfo, PixelFormat, FrameBuffer, BootInfo};

use crate::io::screen;

lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen {
        framebuffer: &mut [0; 0],
        backbuffer: &mut [0; 0],
        clearbuffer: &mut [0; 0],
        byte_len: 0,
        width: 0,
        height: 0,
        pixel_format: PixelFormat::Rgb,
        bytes_per_pixel: 0,
        stride: 0,
    });
}

pub struct Screen {
    framebuffer: &'static mut [u8], //[r, g, b, a, r, g, b, a]
    backbuffer: &'static mut [u8],
    clearbuffer: &'static mut [u8],
    byte_len: usize,
    width: usize,
    height: usize,
    pixel_format: PixelFormat,
    bytes_per_pixel: usize,
    stride: usize,
}

impl Screen {
    pub fn init(&mut self, framebuffer: &'static mut [u8], info: FrameBufferInfo) {
        self.framebuffer = framebuffer;
        self.byte_len = info.byte_len;
        self.width = info.width;
        self.height = info.height;
        self.pixel_format = info.pixel_format;
        self.bytes_per_pixel = info.bytes_per_pixel;
        self.stride = info.stride;

        let mut back_buffer: Box<[u8]> = vec![0; self.byte_len].into_boxed_slice();
        self.backbuffer = Box::leak(back_buffer);

        let mut clear_buffer: Box<[u8]> = vec![0; self.byte_len].into_boxed_slice();
        self.clearbuffer = Box::leak(clear_buffer);
    }

    pub fn clear(&mut self, colour: u32) {
        let rgba = screen::hex_to_rgba(colour);
        let packed_colour = self.pack_colour(rgba.0, rgba.1, rgba.2, 255); // can't have any alpha!

        let bytes_per_pixel = self.bytes_per_pixel;
        let fill_value = packed_colour[..bytes_per_pixel].repeat(self.stride * self.height);

        self.backbuffer[..].copy_from_slice(&fill_value);
    }

    pub fn pixel_fast(&mut self, x: usize, y: usize, colour: u32) {
        if x >= 0 && x <= self.width-1 && y >= 0 && y <= self.height-1 {
            let pixel_offset = y * self.stride + x;
            let rgba = screen::hex_to_rgba(colour);
            let c = self.pack_colour(rgba.0, rgba.1, rgba.2, rgba.3);

            let bytes_per_pixel = self.bytes_per_pixel;
            let byte_offset = pixel_offset * bytes_per_pixel;

            self.backbuffer[byte_offset..(byte_offset + bytes_per_pixel)].copy_from_slice(&c[..bytes_per_pixel]);
            let _ = unsafe { ptr::read_volatile(&self.backbuffer[byte_offset]) };
        }
    }

    pub fn pixel(&mut self, x: usize, y: usize, colour: u32) {
        if x >= 0 && x <= self.width - 1 && y >= 0 && y <= self.height - 1 {
            let pixel_offset = y * self.stride + x;
            let rgba = screen::hex_to_rgba(colour);
    
            let existing_rgba = self.unpack_colour(&self.backbuffer[pixel_offset * self.bytes_per_pixel..]);
    
            let blended_rgba = self.blend_color(rgba, existing_rgba);
    
            let c = self.pack_colour(blended_rgba.0, blended_rgba.1, blended_rgba.2, blended_rgba.3);
    
            let bytes_per_pixel = self.bytes_per_pixel;
            let byte_offset = pixel_offset * bytes_per_pixel;
    
            self.backbuffer[byte_offset..(byte_offset + bytes_per_pixel)].copy_from_slice(&c[..bytes_per_pixel]);
            let _ = unsafe { ptr::read_volatile(&self.backbuffer[byte_offset]) };
        }
    }

    pub fn pack_colour(&mut self, r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
        let colour = match self.pixel_format {
            PixelFormat::Rgb => [r, g, b, a],
            PixelFormat::Bgr => [b, g, r, a],
            PixelFormat::U8 => [if r > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        colour
    }

    fn unpack_colour(&self, rgba: &[u8]) -> (u8, u8, u8, u8) {
        match self.pixel_format {
            PixelFormat::Rgb => (rgba[0], rgba[1], rgba[2], 255),
            PixelFormat::Bgr => (rgba[2], rgba[1], rgba[0], 255),
            _ => panic!("Invalid pixel format for unpacking color."),
        }
    }

    fn blend_color(&self, new_rgba: (u8, u8, u8, u8), existing_rgba: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
        let alpha = new_rgba.3 as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;
    
        let blended_r = (new_rgba.0 as f32 * alpha + existing_rgba.0 as f32 * inv_alpha) as u8;
        let blended_g = (new_rgba.1 as f32 * alpha + existing_rgba.1 as f32 * inv_alpha) as u8;
        let blended_b = (new_rgba.2 as f32 * alpha + existing_rgba.2 as f32 * inv_alpha) as u8;
        let blended_a = (new_rgba.3 as f32 * alpha + existing_rgba.3 as f32 * inv_alpha) as u8;
    
        (blended_r, blended_g, blended_b, blended_a)
    }

    pub fn flip(&mut self) {
        self.framebuffer.copy_from_slice(&self.backbuffer[0..self.byte_len]);
    }

    // pub fn shift_y(&mut self, shift_amount: usize) {
    //     let row_length = self.byte_len / self.height;
        
    //     for row in 0..self.height {
    //         let src_start = row * row_length;
    //         let dest_start = ((row + shift_amount) % self.height) * row_length;
    //         let src_slice = &self.backbuffer[src_start..src_start + row_length];
    //         self.backbuffer[dest_start..dest_start + row_length].copy_from_slice(src_slice);
    //     }
    // }

    // pub fn shift_y(&mut self, shift_amount: u8) {
    //     let row_length = self.byte_len / self.height;
        
    //     let mut temp_buffer = Vec::with_capacity(self.byte_len);
    //     temp_buffer.extend_from_slice(&self.backbuffer[0..self.byte_len]);
    
    //     for row in 0..self.height-1 {
    //         let src_start = row * row_length;
    //         let dest_start = ((row + shift_amount) % self.height) * row_length;
    //         let src_slice = &temp_buffer[src_start..src_start + row_length];
    //         self.backbuffer[dest_start..dest_start + row_length].copy_from_slice(src_slice);
    //     }

    // }

    pub fn shift_y(&mut self, amount: usize) {
        let row_length = self.byte_len / self.height;
        
        let mut new_buffer = vec![0; self.byte_len];
        new_buffer.copy_from_slice(&self.backbuffer[0..self.byte_len]);

        self.backbuffer.copy_from_slice(&self.backbuffer[0..self.byte_len]);
    }
    
}
