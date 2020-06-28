mod error;

pub use self::error::*;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use png::{BitDepth, ColorType};

use embercore::tme;

pub fn load_tileset(source: &str) -> Result<tme::Tileset> {
    let reader = BufReader::new(File::open(source).context(source.to_owned())?);

    let tileset = serde_json::from_reader(reader).context(source.to_owned())?;

    Ok(tileset)
}

pub fn load_texture(folder: &str, source: &str) -> Result<(png::OutputInfo, Vec<u8>)> {
    let reader = BufReader::new(File::open(Path::new(folder).join(source)).context(source.to_owned())?);

    let decoder = png::Decoder::new(reader);
    let (info, mut reader) = decoder.read_info().context(source.to_owned())?;

    if !matches!(info.color_type, ColorType::RGB | ColorType::RGBA) {
        return Err(Error::UnsupportedColorType(info.color_type)).context(source.to_owned());
    }

    if !matches!(info.bit_depth, BitDepth::Eight) {
        return Err(Error::UnsupportedBitDepth(info.bit_depth)).context(source.to_owned());
    }

    let mut result = Vec::new();
    result.resize((info.width * info.height) as usize * info.color_type.samples(), 0);
    reader.next_frame(&mut result).context(source.to_owned())?;

    Ok((info, result))
}
