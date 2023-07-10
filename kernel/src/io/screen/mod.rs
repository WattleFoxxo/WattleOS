pub mod framebuffer;
pub mod console;

use bootloader_api::info::{FrameBufferInfo, PixelFormat, FrameBuffer, BootInfo};
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};
use font_constants::BACKUP_CHAR;

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;

mod font_constants {
    use super::*;
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
    pub const BACKUP_CHAR: char = '�';
    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}


pub fn rgba_to_hex(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    ((red as u32) << 24) | ((green as u32) << 16) | ((blue as u32) << 8) | alpha as u32
}
    
pub fn hex_to_rgba(int_value: u32) -> (u8, u8, u8, u8) {
    let red = (int_value >> 24) as u8;
    let green = ((int_value >> 16) & 0xFF) as u8;
    let blue = ((int_value >> 8) & 0xFF) as u8;
    let alpha = (int_value & 0xFF) as u8;
    (red, green, blue, alpha)
}


pub fn init(boot_info: &'static BootInfo) {
    let boot_info = boot_info as *const BootInfo as *mut BootInfo;
    let boot_info_mut = unsafe { &mut *(boot_info) };
    let framebuffer = boot_info_mut.framebuffer.as_mut().unwrap();
    let info = framebuffer.info().clone();
        
    framebuffer::SCREEN.lock().init(framebuffer.buffer_mut(), info);
    framebuffer::SCREEN.lock().clear(0x00_00_00_00);
    framebuffer::SCREEN.lock().flip();
    use crate::println;
    println!("{}", font_constants::CHAR_RASTER_WIDTH); //7, 142
    println!("{}", font_constants::CHAR_RASTER_HEIGHT.val() + 1); //17, 47
    //char();
}

pub fn flip() {
    framebuffer::SCREEN.lock().flip();
}

pub fn clear(colour: u32) {
    framebuffer::SCREEN.lock().clear(colour);
}

pub fn pixel(x: usize, y: usize, colour: u32) {
    framebuffer::SCREEN.lock().pixel(x, y, colour);
}

pub fn rectangle(x: usize, y: usize, w: usize, h: usize, colour: u32) {
    for i in 0..w {
        for j in 0..h {
            pixel(i+x, j+y, colour);
        }
    }
}

pub fn char(x: usize, y: usize, fg_colour: u32, bg_colour: u32, c: char) {
    let rendered_char = get_char_raster(c);
    let fg_rgba = hex_to_rgba(fg_colour);
    let bg_rgba = hex_to_rgba(bg_colour);
    //pixel(x, y, 0xFF_FF_FF_FF);
    for (i, row) in rendered_char.raster().iter().enumerate() {
        for (j, byte) in row.iter().enumerate() {
            let alpha = (*byte) as f32 / 255.0;
            let blend_colour = (
                (fg_rgba.0 as f32 * alpha + bg_rgba.0 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.1 as f32 * alpha + bg_rgba.1 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.2 as f32 * alpha + bg_rgba.2 as f32 * (1.0 - alpha)) as u8,
                fg_rgba.3,
            );
            framebuffer::SCREEN.lock().pixel_fast(x + j, y + i, rgba_to_hex(blend_colour.0, blend_colour.1, blend_colour.2, blend_colour.3));
        }
    }
}

pub fn char_slow(x: usize, y: usize, fg_colour: u32, bg_colour: u32, c: char) {
    let rendered_char = get_char_raster(c);
    let fg_rgba = hex_to_rgba(fg_colour);
    let bg_rgba = hex_to_rgba(bg_colour);
    //pixel(x, y, 0xFF_FF_FF_FF);
    for (i, row) in rendered_char.raster().iter().enumerate() {
        for (j, byte) in row.iter().enumerate() {
            let alpha = (*byte) as f32 / 255.0;
            let blend_colour = (
                (fg_rgba.0 as f32 * alpha + bg_rgba.0 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.1 as f32 * alpha + bg_rgba.1 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.2 as f32 * alpha + bg_rgba.2 as f32 * (1.0 - alpha)) as u8,
                fg_rgba.3,
            );
            framebuffer::SCREEN.lock().pixel_fast(x + j, y + i, rgba_to_hex(blend_colour.0, blend_colour.1, blend_colour.2, blend_colour.3));
        }
    }
}

pub fn char_old(x: usize, y: usize, fg_colour: u32, bg_colour: u32, c: char) {
    let rendered_char = get_char_raster(c);
    let fg_rgba = hex_to_rgba(fg_colour);
    let bg_rgba = hex_to_rgba(bg_colour);
    
    for (i, row) in rendered_char.raster().iter().enumerate() {
        for (j, byte) in row.iter().enumerate() {
            let alpha = (*byte) as f32 / 255.0;
            let blended_rgba = (
                (fg_rgba.0 as f32 * alpha + bg_rgba.0 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.1 as f32 * alpha + bg_rgba.1 as f32 * (1.0 - alpha)) as u8,
                (fg_rgba.2 as f32 * alpha + bg_rgba.2 as f32 * (1.0 - alpha)) as u8,
                fg_rgba.3,
            );
            pixel(x + j, y + i, rgba_to_hex(blended_rgba.0, blended_rgba.1, blended_rgba.2, blended_rgba.3));
        }
    }
}

pub fn shift_y(shift_amount: usize) {
    framebuffer::SCREEN.lock().shift_y(shift_amount);
}

fn pack_colour(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
    [r, g, b, a]
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

fn blend_color(new_rgba: (u8, u8, u8, u8), existing_rgba: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let alpha = new_rgba.3 as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let blended_r = (new_rgba.0 as f32 * alpha + existing_rgba.0 as f32 * inv_alpha) as u8;
    let blended_g = (new_rgba.1 as f32 * alpha + existing_rgba.1 as f32 * inv_alpha) as u8;
    let blended_b = (new_rgba.2 as f32 * alpha + existing_rgba.2 as f32 * inv_alpha) as u8;
    let blended_a = (new_rgba.3 as f32 * alpha + existing_rgba.3 as f32 * inv_alpha) as u8;

    (blended_r, blended_g, blended_b, blended_a)
}