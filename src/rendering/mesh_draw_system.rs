use vulkano::buffer::{CpuBufferPool, TypedBufferAccess};

use crate::rendering::prelude::*;

pub struct MeshDrawSystem {
    queue: Arc<Queue>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    world_uniform_buffer_pool: CpuBufferPool<vertex_shader::ty::WorldData>,
    world_descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl MeshDrawSystem {
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

    pub fn update_view<V>(&mut self, view_data_source: &V)
    where
        V: ViewDataSource,
    {
        self.world_descriptor_set =
            view_data_source.create_descriptor_set(self.pipeline.as_ref(), &mut self.world_uniform_buffer_pool);
    }

    pub fn draw<D>(&self, dynamic_state: &DynamicState, drawable: &D, mesh_state: &MeshState) -> AutoCommandBuffer
    where
        D: DrawableDataSource,
    {
        let push_constants: vertex_shader::ty::MeshData = mesh_state.into();

        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap()
        .draw_indexed(
            self.pipeline.clone(),
            dynamic_state,
            vec![drawable.vertex_buffer()],
            drawable.index_buffer(),
            self.world_descriptor_set.clone(),
            push_constants,
        )
        .unwrap()
        .build()
        .unwrap()
    }
}

pub trait DrawableDataSource {
    type VertexBuffer: TypedBufferAccess<Content = [Vertex]> + Send + Sync + 'static;
    type IndexBuffer: TypedBufferAccess<Content = [u32]> + Send + Sync + 'static;

    fn vertex_buffer(&self) -> Arc<Self::VertexBuffer>;
    fn index_buffer(&self) -> Arc<Self::IndexBuffer>;
}

pub struct SimpleMesh {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl SimpleMesh {
    pub fn new(queue: Arc<Queue>, path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader).unwrap();

        let mesh = gltf.meshes().next().unwrap();
        let primitive = mesh.primitives().next().unwrap();

        let buffer = gltf.blob.as_ref().unwrap();

        let reader = primitive.reader(|_| Some(buffer));

        let vertex_buffer = CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(), false, {
            reader
                .read_positions()
                .and_then(|positions_iter| reader.read_normals().map(|normals_iter| (positions_iter, normals_iter)))
                .map(|(positions_iter, normals_iter)| {
                    positions_iter.zip(normals_iter).map(|(position, normal)| Vertex {
                        position: [position[0], -position[2], position[1]],
                        normal: [normal[0], -normal[2], normal[1]],
                    })
                })
                .unwrap()
        })
        .expect("Failed to create vertex buffer");

        let index_buffer = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::all(),
            false,
            match reader.read_indices().unwrap() {
                gltf::mesh::util::ReadIndices::U8(iter) => {
                    itertools::Either::Left(itertools::Either::Left(iter.map(|index| index as u32)))
                }
                gltf::mesh::util::ReadIndices::U16(iter) => {
                    itertools::Either::Left(itertools::Either::Right(iter.map(|index| index as u32)))
                }
                gltf::mesh::util::ReadIndices::U32(iter) => itertools::Either::Right(iter),
            },
        )
        .expect("Failed to create index buffer");

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

impl DrawableDataSource for SimpleMesh {
    type VertexBuffer = CpuAccessibleBuffer<[Vertex]>;
    type IndexBuffer = CpuAccessibleBuffer<[u32]>;

    fn vertex_buffer(&self) -> Arc<Self::VertexBuffer> {
        self.vertex_buffer.clone()
    }

    fn index_buffer(&self) -> Arc<Self::IndexBuffer> {
        self.index_buffer.clone()
    }
}

pub trait ViewDataSource {
    fn view(&self) -> glm::Mat4;
    fn projection(&self) -> glm::Mat4;
}

trait ViewDescriptorSetFactory {
    fn create_descriptor_set(
        &self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
        uniform_buffer_pool: &mut CpuBufferPool<vertex_shader::ty::WorldData>,
    ) -> Arc<dyn DescriptorSet + Send + Sync>;
}

impl<T> ViewDescriptorSetFactory for T
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
