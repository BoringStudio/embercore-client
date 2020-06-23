mod error;

pub use self::error::*;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use image::{GenericImageView, RgbaImage};

use embercore::tme;

pub fn load_tileset(source: &str) -> Result<tme::Tileset> {
    let reader = BufReader::new(
        File::open(source)
            .map_err(Error::UnableToOpenFile)
            .context(source.to_owned())?,
    );

    let tileset = serde_json::from_reader(reader)
        .map_err(Error::UnableToParse)
        .context(source.to_owned())?;

    Ok(tileset)
}

pub fn load_texture(folder: &str, source: &str) -> Result<RgbaImage> {
    let texture = image::open(Path::new(folder).join(source))
        .map_err(Error::UnableToLoadTexture)
        .context(source.to_owned())?;

    Ok(texture
        .as_rgba8()
        .ok_or(Error::BadImageColor)
        .context(source.to_owned())?
        .to_owned())
}
