use crate::rendering::prelude::*;
use crate::rendering::screen_quad::*;
use crate::rendering::utils::DescriptorSetFactory;

pub struct ComposingSubsystem {
    queue: Arc<Queue>,
    vertex_buffer: Arc<ScreenQuadVertexBuffer>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl ComposingSubsystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>, screen_quad: &ScreenQuad, input: ComposingSystemInput) -> Self
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        let fragment_shader =
            fragment_shader::Shader::load(queue.device().clone()).expect("Failed to create fragment shader module");

        let vertex_buffer = screen_quad.vertex_buffer();

        let pipeline = Arc::new(
            screen_quad
                .start_graphics_pipeline()
                .fragment_shader(fragment_shader.main_entry_point(), ())
                .render_pass(subpass)
                .build(queue.device().clone())
                .unwrap(),
        );

        let descriptor_set = input.create_descriptor_set(pipeline.as_ref());

        Self {
            queue,
            vertex_buffer,
            pipeline,
            descriptor_set,
        }
    }

    pub fn update_input(&mut self, input: ComposingSystemInput) {
        self.descriptor_set = input.create_descriptor_set(self.pipeline.as_ref());
    }

    pub fn draw(&self, dynamic_state: &DynamicState) -> AutoCommandBuffer {
        let mut command_buffer = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        command_buffer
            .draw(
                self.pipeline.clone(),
                dynamic_state,
                vec![self.vertex_buffer.clone()],
                self.descriptor_set.clone(),
                (),
            )
            .unwrap();

        command_buffer.build().unwrap()
    }
}

pub struct ComposingSystemInput {
    pub diffuse: Arc<AttachmentImage>,
    pub depth: Arc<AttachmentImage>,
}

impl DescriptorSetFactory for ComposingSystemInput {
    fn create_descriptor_set(
        self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let layout = pipeline.descriptor_set_layout(0).unwrap();
        Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_image(self.diffuse)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/compose.frag"
    }
}