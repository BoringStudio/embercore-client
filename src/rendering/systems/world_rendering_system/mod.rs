mod drawable_data_source;
mod view_data_source;

pub use self::drawable_data_source::*;
pub use self::view_data_source::*;

use crate::rendering::prelude::*;

pub struct WorldRenderingSystem {
    queue: Arc<Queue>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    world_uniform_buffer_pool: CpuBufferPool<vertex_shader::ty::WorldData>,
    world_descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl WorldRenderingSystem {
    pub fn new<R, V>(queue: Arc<Queue>, subpass: Subpass<R>, view_data_source: &V) -> Self
    where
        R: RenderPassAbstract + Send + Sync + 'static,
        V: ViewDataSource,
    {
        let pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync> = {
            let vertex_shader =
                vertex_shader::Shader::load(queue.device().clone()).expect("Failed to create vertex shader module");
            let fragment_shader =
                fragment_shader::Shader::load(queue.device().clone()).expect("Failed to create fragment shader module");

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vertex_shader.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fragment_shader.main_entry_point(), ())
                    .depth_stencil(DepthStencil {
                        depth_compare: Compare::Less,
                        depth_write: true,
                        depth_bounds_test: DepthBounds::Disabled,
                        stencil_front: Stencil {
                            compare: Compare::Always,
                            pass_op: StencilOp::Replace,
                            fail_op: StencilOp::Replace,
                            depth_fail_op: StencilOp::Replace,
                            compare_mask: Some(0x80),
                            write_mask: Some(0xff),
                            reference: Some(0x80),
                        },
                        stencil_back: Stencil {
                            compare: Compare::Always,
                            pass_op: StencilOp::Replace,
                            fail_op: StencilOp::Keep,
                            depth_fail_op: StencilOp::Keep,
                            compare_mask: Some(0x80),
                            write_mask: Some(0xff),
                            reference: Some(0x80),
                        },
                    })
                    .render_pass(subpass)
                    .build(queue.device().clone())
                    .unwrap(),
            ) as Arc<_>
        };

        let mut world_uniform_buffer_pool =
            CpuBufferPool::<vertex_shader::ty::WorldData>::new(queue.device().clone(), BufferUsage::all());

        let world_descriptor_set =
            view_data_source.create_descriptor_set(pipeline.as_ref(), &mut world_uniform_buffer_pool);

        Self {
            queue,
            pipeline,
            world_uniform_buffer_pool,
            world_descriptor_set,
        }
    }

    #[allow(dead_code)]
    pub fn update_view<V>(&mut self, view_data_source: &V)
    where
        V: ViewDataSource,
    {
        self.world_descriptor_set =
            view_data_source.create_descriptor_set(self.pipeline.as_ref(), &mut self.world_uniform_buffer_pool);
    }

    #[allow(dead_code)]
    pub fn draw<D>(&self, dynamic_state: &DynamicState, drawable: &D, mesh_state: &MeshState) -> AutoCommandBuffer
    where
        D: DrawableDataSource,
    {
        let push_constants: vertex_shader::ty::MeshData = mesh_state.into();

        let mut command_buffer = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        command_buffer
            .draw_indexed(
                self.pipeline.clone(),
                dynamic_state,
                vec![drawable.vertex_buffer()],
                drawable.index_buffer(),
                self.world_descriptor_set.clone(),
                push_constants,
            )
            .unwrap();

        command_buffer.build().unwrap()
    }
}

#[derive(Clone)]
pub struct MeshState {
    pub transform: glm::Mat4,
}

impl From<&MeshState> for vertex_shader::ty::MeshData {
    fn from(data: &MeshState) -> Self {
        Self {
            transform: data.transform.clone().into(),
        }
    }
}

impl Default for MeshState {
    fn default() -> Self {
        Self {
            transform: glm::Mat4::identity(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position, normal);

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/mesh.vert"
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/mesh.frag"
    }
}
