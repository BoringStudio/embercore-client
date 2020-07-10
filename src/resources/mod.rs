mod error;

pub use self::error::*;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use png::{BitDepth, ColorType};

use embercore::tme;

pub fn load_json<T>(path: &PathBuf) -> Result<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let source = || path.to_string_lossy().to_string();

    let reader = BufReader::new(File::open(path).with_context(source)?);
    serde_json::from_reader(reader).with_context(source)
}

pub fn load_texture(path: &PathBuf) -> Result<(png::OutputInfo, Vec<u8>)> {
    let source = || path.to_string_lossy().to_string();

    let reader = BufReader::new(File::open(path).with_context(source)?);

    let decoder = png::Decoder::new(reader);
    let (info, mut reader) = decoder.read_info().with_context(source)?;

    if !matches!(info.color_type, ColorType::RGB | ColorType::RGBA) {
        return Err(Error::UnsupportedColorType(info.color_type)).with_context(source);
    }

    if !matches!(info.bit_depth, BitDepth::Eight) {
        return Err(Error::UnsupportedBitDepth(info.bit_depth)).with_context(source);
    }

    let mut result = Vec::new();
    result.resize((info.width * info.height) as usize * info.color_type.samples(), 0);
    reader.next_frame(&mut result).with_context(source)?;

    Ok((info, result))
}
