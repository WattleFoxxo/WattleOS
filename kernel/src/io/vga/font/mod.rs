pub mod cozette;

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

pub struct Config {
	width: usize,
	height: usize,
	chars: usize,
}