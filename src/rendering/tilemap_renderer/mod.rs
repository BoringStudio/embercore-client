mod drawable_data_source;
mod tileset_data;
mod view_data_source;

use anyhow::Result;

pub use self::drawable_data_source::*;
pub use self::tileset_data::*;
pub use self::view_data_source::*;

use crate::rendering::prelude::*;
use crate::rendering::utils::*;

pub struct TileMapRenderer {
    queue: Arc<Queue>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    world_uniform_buffer_pool: CpuBufferPool<vertex_shader::ty::WorldData>,
    world_descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    tileset_uniform_buffer_pool: CpuBufferPool<fragment_shader::ty::TileSetInfo>,
    tileset_descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl TileMapRenderer {
    pub fn new<R, V>(queue: Arc<Queue>, subpass: Subpass<R>, view_data_source: &V) -> Result<Self>
    where
        R: RenderPassAbstract + Send + Sync + 'static,
        V: ViewDataSource,
    {
        let pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync> = {
            let vertex_shader = vertex_shader::Shader::load(queue.device().clone())?;
            let fragment_shader = fragment_shader::Shader::load(queue.device().clone())?;

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vertex_shader.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fragment_shader.main_entry_point(), ())
                    .depth_stencil_simple_depth()
                    .render_pass(subpass)
                    .build(queue.device().clone())?,
            ) as Arc<_>
        };

        let mut world_uniform_buffer_pool =
            CpuBufferPool::<vertex_shader::ty::WorldData>::new(queue.device().clone(), BufferUsage::all());
        let mut tileset_uniform_buffer_pool =
            CpuBufferPool::<fragment_shader::ty::TileSetInfo>::new(queue.device().clone(), BufferUsage::all());

        let world_descriptor_set =
            view_data_source.create_descriptor_set(pipeline.as_ref(), &mut world_uniform_buffer_pool)?;

        let tileset_descriptor_set = TileSetInfo::new_empty(queue.clone())?
            .create_descriptor_set(pipeline.as_ref(), &mut tileset_uniform_buffer_pool)?;

        Ok(Self {
            queue,
            pipeline,
            world_uniform_buffer_pool,
            world_descriptor_set,
            tileset_uniform_buffer_pool,
            tileset_descriptor_set,
        })
    }

    pub fn update_view<V>(&mut self, view_data_source: &V) -> Result<()>
    where
        V: ViewDataSource,
    {
        self.world_descriptor_set =
            view_data_source.create_descriptor_set(self.pipeline.as_ref(), &mut self.world_uniform_buffer_pool)?;
        Ok(())
    }

    pub fn update_tileset(&mut self, tileset: TileSetInfo) -> Result<()> {
        self.tileset_descriptor_set =
            tileset.create_descriptor_set(self.pipeline.as_ref(), &mut self.tileset_uniform_buffer_pool)?;
        Ok(())
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
                vec![drawable.vertex_buffer().clone()],
                drawable.index_buffer().clone(),
                (self.world_descriptor_set.clone(), self.tileset_descriptor_set.clone()),
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
    pub texture_coords: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, texture_coords);

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/tile.vert"
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/tile.frag"
    }
}
