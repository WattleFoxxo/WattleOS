pub mod framebuffer;
pub mod font;

use bootloader_api::info::{FrameBufferInfo, PixelFormat, FrameBuffer, BootInfo};


/// Initialize the screen
pub fn init(boot_info: &'static BootInfo) {
    let boot_info = boot_info as *const BootInfo as *mut BootInfo;
    let boot_info_mut = unsafe { &mut *(boot_info) };
    let framebuffer = boot_info_mut.framebuffer.as_mut().unwrap();
    let info = framebuffer.info().clone();
        
    framebuffer::VGA.lock().init(framebuffer.buffer_mut(), info);
    framebuffer::VGA.lock().clear(0x00_00_00_00);
    framebuffer::VGA.lock().flip();
}

/// Clear the screen
pub fn clear(colour: u32) {
    framebuffer::VGA.lock().clear(colour);
}

/// Draw pixel
pub fn pixel(x: usize, y: usize, colour: u32) {
    framebuffer::VGA.lock().pixel(x, y, colour);
}

/// Draw pixel without alpha
pub fn pixel_fast(x: usize, y: usize, colour: u32) {
    framebuffer::VGA.lock().pixel_fast(x, y, colour);
}

/// Shift the framebuffer by <amount> on the y axis
pub fn shift_y(amount: usize) {
    framebuffer::VGA.lock().shift_y(amount);
}

/// Flip the double buffer
pub fn flip() {
    framebuffer::VGA.lock().flip();
}

pub fn rect(x: usize, y: usize, w: usize, h: usize, colour: u32) {
    for j in 0..w {
        for k in 0..h {
            pixel(x+j, y+k, colour);
        }
    }
}

pub fn char(x: usize, y: usize, fg_colour: u32, bg_colour: u32, c: char) {
    
}

pub fn char_bitmap(x: usize, y: usize, size: usize, fg_colour: u32, bg_colour: u32, c: char) {
    let mut x_offset: usize = 0;
    let mut y_offset: usize = 0;
    for j in &font::cozette::DATA[c as usize] {
        y_offset += 1;
        for bit in 0..8 {
            rect(x+(x_offset*size), y+(y_offset*size), size, size, bg_colour);
            match *j & 1 << (bit*-1)+8 {
                0 => {
                },
                _ => {
                    rect(x+(x_offset*size), y+(y_offset*size), size, size, fg_colour);
                },
            }
            x_offset += 1;
        }
        x_offset = 0;
    }
}

// Get functions

pub fn width() -> usize {
    framebuffer::VGA.lock().width()
}

pub fn height() -> usize {
    framebuffer::VGA.lock().height()
}

// Helpers

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

pub fn blend_colour(new_rgba: (u8, u8, u8, u8), existing_rgba: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let alpha = new_rgba.3 as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let blended_r = (new_rgba.0 as f32 * alpha + existing_rgba.0 as f32 * inv_alpha) as u8;
    let blended_g = (new_rgba.1 as f32 * alpha + existing_rgba.1 as f32 * inv_alpha) as u8;
    let blended_b = (new_rgba.2 as f32 * alpha + existing_rgba.2 as f32 * inv_alpha) as u8;
    let blended_a = (new_rgba.3 as f32 * alpha + existing_rgba.3 as f32 * inv_alpha) as u8;

    (blended_r, blended_g, blended_b, blended_a)
}