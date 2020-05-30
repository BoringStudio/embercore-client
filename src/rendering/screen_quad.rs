use vulkano::pipeline::shader::EmptyEntryPointDummy;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::GraphicsPipelineBuilder;

use super::prelude::*;

pub struct ScreenQuad {
    vertex_buffer: Arc<ScreenQuadVertexBuffer>,
    vertex_shader: vertex_shader::Shader,
}

impl ScreenQuad {
    pub fn new(queue: Arc<Queue>) -> Self {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::all(),
            false,
            ScreenVertex::quad().iter().cloned(),
        )
        .expect("Failed to create screen vertex buffer");

        let vertex_shader =
            vertex_shader::Shader::load(queue.device().clone()).expect("Failed to create screen vertex shader module");

        Self {
            vertex_buffer,
            vertex_shader,
        }
    }

    #[inline]
    pub fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[ScreenVertex]>> {
        self.vertex_buffer.clone()
    }

    #[inline]
    pub fn start_graphics_pipeline(
        &self,
    ) -> GraphicsPipelineBuilder<
        SingleBufferDefinition<ScreenVertex>,
        vulkano::pipeline::shader::GraphicsEntryPoint<
            (),
            vertex_shader::MainInput,
            vertex_shader::MainOutput,
            vertex_shader::Layout,
        >,
        (),
        EmptyEntryPointDummy,
        (),
        EmptyEntryPointDummy,
        (),
        EmptyEntryPointDummy,
        (),
        EmptyEntryPointDummy,
        (),
        (),
    > {
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<ScreenVertex>()
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .triangle_fan()
            .viewports_dynamic_scissors_irrelevant(1)
    }
}

pub type ScreenQuadVertexBuffer = CpuAccessibleBuffer<[ScreenVertex]>;

#[derive(Default, Debug, Clone)]
pub struct ScreenVertex {
    pub position: [f32; 2],
}

vulkano::impl_vertex!(ScreenVertex, position);

impl ScreenVertex {
    pub fn quad() -> [Self; 4] {
        [
            ScreenVertex { position: [-1.0, -1.0] },
            ScreenVertex { position: [1.0, -1.0] },
            ScreenVertex { position: [1.0, 1.0] },
            ScreenVertex { position: [-1.0, 1.0] },
        ]
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/screen.vert"
    }
}
