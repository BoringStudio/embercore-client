use super::fragment_shader;
use crate::rendering::prelude::*;
use crate::rendering::utils::UniformDescriptorSetFactory;
use crate::rendering::utils::*;

pub struct TileSetInfo {
    pub image: Arc<ImmutableImage<Format>>,
    pub size: [i32; 2],
}

impl TileSetInfo {
    pub fn new_empty(queue: Arc<Queue>) -> Result<Self> {
        Ok(Self {
            image: rgba_null_texture(queue)?,
            size: [1, 1],
        })
    }
}

impl UniformDescriptorSetFactory<fragment_shader::ty::TileSetInfo> for TileSetInfo {
    fn create_descriptor_set(
        &self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
        uniform_buffer_pool: &mut CpuBufferPool<fragment_shader::ty::TileSetInfo>,
    ) -> Result<Arc<dyn DescriptorSet + Send + Sync>> {
        let uniform_data = fragment_shader::ty::TileSetInfo { size: self.size };

        let uniform_buffer = uniform_buffer_pool.next(uniform_data)?;
        let layout = pipeline.descriptor_set_layout(1).unwrap();
        Ok(Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_sampled_image(self.image.clone(), pixel_sampler(pipeline.device().clone())?)?
                .add_buffer(uniform_buffer)?
                .build()?,
        ))
    }
}
