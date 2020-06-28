use super::vertex_shader;
use crate::rendering::prelude::*;
use crate::rendering::utils::UniformDescriptorSetFactory;

pub trait ViewDataSource {
    fn view(&self) -> glm::Mat4;
    fn projection(&self) -> glm::Mat4;
}

pub struct IdentityViewDataSource;

impl ViewDataSource for IdentityViewDataSource {
    fn view(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn projection(&self) -> glm::Mat4 {
        glm::identity()
    }
}

impl<T> UniformDescriptorSetFactory<vertex_shader::ty::WorldData> for T
where
    T: ViewDataSource,
{
    fn create_descriptor_set(
        &self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
        uniform_buffer_pool: &mut CpuBufferPool<vertex_shader::ty::WorldData>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let uniform_data = vertex_shader::ty::WorldData {
            view: self.view().into(),
            projection: self.projection().into(),
        };

        let uniform_buffer = uniform_buffer_pool.next(uniform_data).unwrap();
        let layout = pipeline.descriptor_set_layout(0).unwrap();
        Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_buffer(uniform_buffer)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
