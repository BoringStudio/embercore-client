use once_cell::sync::OnceCell;

pub fn create_rgba_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::Extent3d) {
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };

    (
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        }),
        texture_extent,
    )
}

#[allow(dead_code)]
pub fn pixel_sampler(device: &wgpu::Device) -> &wgpu::Sampler {
    NEAREST_SAMPLER.get_or_init(|| {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    })
}

#[allow(dead_code)]
pub fn rgba_null_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> &'static wgpu::TextureView {
    RGBA_NULL_TEXTURE.get_or_init(|| {
        let (texture, texture_extent) = create_rgba_texture(device, 1, 1);

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &[255u8, 0, 255, 255],
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4,
                rows_per_image: 0,
            },
            texture_extent,
        );

        texture.create_default_view()
    })
}

static NEAREST_SAMPLER: OnceCell<wgpu::Sampler> = OnceCell::new();
static RGBA_NULL_TEXTURE: OnceCell<wgpu::TextureView> = OnceCell::new();
