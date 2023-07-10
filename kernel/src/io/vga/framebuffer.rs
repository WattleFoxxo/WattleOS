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

use crate::io::vga;

lazy_static! {
    pub static ref VGA: Mutex<Vga> = Mutex::new(Vga {
        frontbuffer: &mut [0; 0],
        backbuffer: &mut [0; 0],
        info: FrameBufferInfo {
            byte_len: 0,
            width: 0,
            height: 0,
            pixel_format: PixelFormat::Rgb,
            bytes_per_pixel: 0,
            stride: 0,
        },
    });
}

pub struct Vga {
    frontbuffer: &'static mut [u8],
    backbuffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Vga {
    pub fn init(&mut self, framebuffer: &'static mut [u8], info: FrameBufferInfo) {
        self.info = info;
        self.frontbuffer = framebuffer;

        let mut back_buffer: Box<[u8]> = vec![0; self.info.byte_len].into_boxed_slice();
        self.backbuffer = Box::leak(back_buffer);

    }


    pub fn clear(&mut self, colour: u32) {
        let rgba = vga::hex_to_rgba(colour);
        let packed_colour = self.pack_colour(rgba.0, rgba.1, rgba.2, 255); // can't have any alpha!

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let fill_value = packed_colour[..bytes_per_pixel].repeat(self.info.stride * self.info.height);

        self.backbuffer[..].copy_from_slice(&fill_value);
    }


    pub fn pixel(&mut self, x: usize, y: usize, colour: u32) {
        if x >= 0 && x <= self.info.width - 1 && y >= 0 && y <= self.info.height - 1 {
            let pixel_offset = y * self.info.stride + x;
            let rgba = vga::hex_to_rgba(colour);
    
            let bytes_per_pixel = self.info.bytes_per_pixel;
            let byte_offset = pixel_offset * bytes_per_pixel;

            let existing_rgba = self.unpack_colour(&self.backbuffer[byte_offset..]);
    
            let blended_rgba = vga::blend_colour(rgba, existing_rgba);
    
            let c = self.pack_colour(blended_rgba.0, blended_rgba.1, blended_rgba.2, blended_rgba.3);
    
            self.backbuffer[byte_offset..(byte_offset + bytes_per_pixel)].copy_from_slice(&c[..bytes_per_pixel]);
            let _ = unsafe { ptr::read_volatile(&self.backbuffer[byte_offset]) };
        }
    }


    pub fn pixel_fast(&mut self, x: usize, y: usize, colour: u32) {
        if x >= 0 && x <= self.info.width-1 && y >= 0 && y <= self.info.height-1 {
            let pixel_offset = y * self.info.stride + x;
            let rgba = vga::hex_to_rgba(colour);
            let c = self.pack_colour(rgba.0, rgba.1, rgba.2, rgba.3);

            let bytes_per_pixel = self.info.bytes_per_pixel;
            let byte_offset = pixel_offset * bytes_per_pixel;

            self.backbuffer[byte_offset..(byte_offset + bytes_per_pixel)].copy_from_slice(&c[..bytes_per_pixel]);
            let _ = unsafe { ptr::read_volatile(&self.backbuffer[byte_offset]) };
        }
    }

    pub fn shift_y_old(&mut self, amount: isize, start: usize, end: usize) {
        let row_length = self.info.byte_len / self.info.height;
        
        let mut temp_buffer = Vec::with_capacity(self.info.byte_len);
        temp_buffer.extend_from_slice(&self.backbuffer[0..self.info.byte_len]);
    
        for row in 0..self.info.height {
            let src_start = row * row_length;
            let dest_start = (((row as isize) + amount + self.info.height as isize) % self.info.height as isize) as usize * row_length;
            let src_slice = &temp_buffer[src_start..src_start + row_length];
            self.backbuffer[dest_start..dest_start + row_length].copy_from_slice(src_slice);
        }
    }
    
    // pub fn shift_y(&mut self, amount: usize, start: usize, end: usize) {
    //     //pub fn flip(&mut self, shift: usize) {
    //         let mut temp_buffer = vec![0; self.info.byte_len];
    //         for y in 0..self.info.height {
    //             let src_start = y * self.info.stride;
    //             let dst_start = ((y + amount) % self.info.height) * self.info.stride;
    //             temp_buffer[dst_start..dst_start + self.info.stride]
    //                 .copy_from_slice(&self.backbuffer[src_start..src_start + self.info.stride]);
    //         }
    //         self.backbuffer.copy_from_slice(&temp_buffer);
    //     //}
    // }

    // pub fn shift_y(&mut self, shift: usize, start: usize, end: usize) {
    //     let row_size = buffer.len() / shift;
    //     let mut temp = vec![0; row_size];
    
    //     for i in 0..shift {
    //         let src_offset = i * row_size;
    //         let dest_offset = (i + 1) * row_size;
    //         temp.copy_from_slice(&buffer[src_offset..src_offset + row_size]);
    //         self.backbuffer[dest_offset..dest_offset + row_size].copy_from_slice(&temp);
    //     }
    // }
    

    pub fn shift_y(&mut self, amount: usize) {
        let row_length = self.info.byte_len / self.info.height;

        let shift_amount = amount * row_length;
        
        let mut new_buffer = vec![0; self.info.byte_len];
        new_buffer[0..self.info.byte_len-shift_amount].copy_from_slice(&self.backbuffer[shift_amount..self.info.byte_len]);

        self.backbuffer.copy_from_slice(&new_buffer[0..self.info.byte_len]);
    }
    

    pub fn flip(&mut self) {
        self.frontbuffer.copy_from_slice(&self.backbuffer[0..self.info.byte_len]);
    }

    // get functions
    pub fn width(&self) -> usize {
        self.info.width
    }

    pub fn height(&self) -> usize {
        self.info.height
    }

    // helper functions
    pub fn pack_colour(&self, r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
        let colour = match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, a],
            PixelFormat::Bgr => [b, g, r, a],
            PixelFormat::U8 => [if r > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        colour
    }

    pub fn unpack_colour(&self, rgba: &[u8]) -> (u8, u8, u8, u8) {
        match self.info.pixel_format {
            PixelFormat::Rgb => (rgba[0], rgba[1], rgba[2], 255),
            PixelFormat::Bgr => (rgba[2], rgba[1], rgba[0], 255),
            _ => panic!("Invalid pixel format for unpacking color."),
        }
    }

}
