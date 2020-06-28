use anyhow::Result;
use once_cell::sync::OnceCell;

use super::prelude::*;

pub trait DescriptorSetFactory {
    fn create_descriptor_set(
        self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
    ) -> Result<Arc<dyn DescriptorSet + Send + Sync>>;
}

pub trait UniformDescriptorSetFactory<T> {
    fn create_descriptor_set(
        &self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
        uniform_buffer_pool: &mut CpuBufferPool<T>,
    ) -> Result<Arc<dyn DescriptorSet + Send + Sync>>;
}

#[allow(dead_code)]
pub fn pixel_sampler(device: Arc<Device>) -> Result<Arc<Sampler>> {
    if let Some(sampler) = NEAREST_SAMPLER.get() {
        return Ok(sampler.clone());
    }

    let sampler = Sampler::new(
        device,
        Filter::Nearest,
        Filter::Nearest,
        MipmapMode::Nearest,
        SamplerAddressMode::MirroredRepeat,
        SamplerAddressMode::MirroredRepeat,
        SamplerAddressMode::MirroredRepeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )?;

    let _ = NEAREST_SAMPLER.set(sampler.clone());
    Ok(sampler)
}

#[allow(dead_code)]
pub fn rgb_null_texture(queue: Arc<Queue>) -> Result<Arc<ImmutableImage<Format>>> {
    if let Some(image) = RGB_NULL_TEXTURE.get() {
        return Ok(image.clone());
    }

    let dimensions = Dimensions::Dim2d { width: 1, height: 1 };

    let (image, image_fut) =
        ImmutableImage::from_iter([255u8, 0, 255].iter().cloned(), dimensions, Format::R8G8B8Srgb, queue)?;

    image_fut.then_signal_fence_and_flush()?.wait(None)?;

    let _ = RGB_NULL_TEXTURE.set(image.clone());
    Ok(image)
}

#[allow(dead_code)]
pub fn rgba_null_texture(queue: Arc<Queue>) -> Result<Arc<ImmutableImage<Format>>> {
    if let Some(image) = RGBA_NULL_TEXTURE.get() {
        return Ok(image.clone());
    }

    let dimensions = Dimensions::Dim2d { width: 1, height: 1 };

    let (image, image_fut) = ImmutableImage::from_iter(
        [255u8, 0, 255, 255].iter().cloned(),
        dimensions,
        Format::R8G8B8A8Srgb,
        queue,
    )?;

    image_fut.then_signal_fence_and_flush()?.wait(None)?;

    let _ = RGBA_NULL_TEXTURE.set(image.clone());
    Ok(image)
}

static NEAREST_SAMPLER: OnceCell<Arc<Sampler>> = OnceCell::new();
static RGB_NULL_TEXTURE: OnceCell<Arc<ImmutableImage<Format>>> = OnceCell::new();
static RGBA_NULL_TEXTURE: OnceCell<Arc<ImmutableImage<Format>>> = OnceCell::new();
