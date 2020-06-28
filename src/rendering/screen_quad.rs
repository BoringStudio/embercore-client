use anyhow::Result;

use super::prelude::*;

pub struct ScreenQuad {
    vertex_buffer: Arc<ScreenQuadVertexBuffer>,
    vertex_shader: vertex_shader::Shader,
}

impl ScreenQuad {
    pub fn new(queue: Arc<Queue>) -> Result<Self> {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::all(),
            false,
            ScreenVertex::quad().iter().cloned(),
        )?;

        let vertex_shader = vertex_shader::Shader::load(queue.device().clone())?;

        Ok(Self {
            vertex_buffer,
            vertex_shader,
        })
    }

    #[inline]
    pub fn vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[ScreenVertex]>> {
        self.vertex_buffer.clone()
    }

    #[inline]
    pub fn start_graphics_pipeline(&self) -> QuadGraphicsPipeline {
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<ScreenVertex>()
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .triangle_fan()
            .viewports_dynamic_scissors_irrelevant(1)
    }
}

pub type ScreenQuadVertexBuffer = CpuAccessibleBuffer<[ScreenVertex]>;

type QuadGraphicsPipeline<'a> = GraphicsPipelineBuilder<
    SingleBufferDefinition<ScreenVertex>,
    vulkano::pipeline::shader::GraphicsEntryPoint<
        'a,
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
>;

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
