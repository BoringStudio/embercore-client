use super::composition_system::*;
use super::prelude::*;
use crate::rendering::screen_quad::ScreenQuad;

pub struct FrameSystem {
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,

    swapchain: Arc<Swapchain<Window>>,
    attachments: Attachments,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    should_recreate_swapchain: bool,
    frame_future: Option<Box<dyn GpuFuture>>,

    composing_system: ComposingSystem,
}

impl FrameSystem {
    pub fn new(surface: Arc<Surface<Window>>, queue: Arc<Queue>) -> Self {
        let dimensions = surface.window().inner_size().into();

        let format;

        let (swapchain, swapchain_images) = {
            let surface_capabilities = surface
                .capabilities(queue.device().physical_device())
                .expect("Failed to get surface capabilities");

            let usage = surface_capabilities.supported_usage_flags;
            let alpha = surface_capabilities.supported_composite_alpha.iter().next().unwrap();
            format = surface_capabilities.supported_formats[0].0;

            Swapchain::new(
                queue.device().clone(),
                surface.clone(),
                surface_capabilities.min_image_count,
                format,
                dimensions,
                1,
                usage,
                SharingMode::Exclusive,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            )
            .expect("Failed to create swapchain")
        };

        let attachments = Attachments::new(queue.device().clone(), dimensions);

        let render_pass = Arc::new(
            vulkano::ordered_passes_renderpass!(queue.device().clone(),
                attachments: {
                    final_color: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    },
                    diffuse: {
                        load: Clear,
                        store: DontCare,
                        format: ImageViewAccess::format(&attachments.diffuse),
                        samples: 1,
                    },
                    normals: {
                        load: Clear,
                        store: DontCare,
                        format: ImageViewAccess::format(&attachments.normals),
                        samples: 1,
                    },
                    light: {
                        load: Clear,
                        store: DontCare,
                        format: ImageViewAccess::format(&attachments.light),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: ImageViewAccess::format(&attachments.depth),
                        samples: 1,
                    }
                },
                passes: [
                    {
                        color: [diffuse, normals],
                        depth_stencil: {depth},
                        input: []
                    },
                    {
                        color: [light],
                        depth_stencil: {depth},
                        input: [diffuse, normals/*, depth*/]
                    },
                    {
                        color: [final_color],
                        depth_stencil: {},
                        input: [diffuse, light, depth]
                    }
                ]
            )
            .unwrap(),
        );

        let mut dynamic_state = DynamicState::none();

        let framebuffers = Self::create_framebuffers(
            dimensions,
            swapchain_images,
            &attachments,
            render_pass.clone(),
            &mut dynamic_state,
        );

        let screen_quad = ScreenQuad::new(queue.clone());

        let composing_subpass = Subpass::from(render_pass.clone(), 2).unwrap();
        let composing_system = ComposingSystem::new(
            queue.clone(),
            composing_subpass,
            &screen_quad,
            attachments.clone().into(),
        );

        let frame_future = Some(Box::new(vulkano::sync::now(queue.device().clone())) as Box<dyn GpuFuture>);

        Self {
            surface,
            queue,
            swapchain,
            attachments,
            dynamic_state,
            render_pass: render_pass as Arc<_>,
            framebuffers,
            should_recreate_swapchain: false,
            frame_future,
            composing_system,
        }
    }

    #[inline]
    pub fn deferred_subpass(&self) -> Subpass<Arc<dyn RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    #[inline]
    pub fn invalidate_swapchain(&mut self) {
        self.should_recreate_swapchain = true;
    }

    pub fn frame(&mut self) -> Option<Frame> {
        self.frame_future.as_mut().unwrap().cleanup_finished();

        if self.should_recreate_swapchain {
            let dimensions = self.surface.window().inner_size().into();
            let (swapchain, swapchain_images) = match self.swapchain.recreate_with_dimensions(dimensions) {
                Ok(result) => result,
                Err(SwapchainCreationError::UnsupportedDimensions) => return None,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

            self.swapchain = swapchain;
            self.attachments = Attachments::new(self.queue.device().clone(), dimensions);
            self.framebuffers = Self::create_framebuffers(
                dimensions,
                swapchain_images,
                &self.attachments,
                self.render_pass.clone(),
                &mut self.dynamic_state,
            );

            self.composing_system.update_input(self.attachments.clone().into());

            self.should_recreate_swapchain = false;
        }

        let (swapchain_image_index, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    self.should_recreate_swapchain = true;
                    return None;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.should_recreate_swapchain = true;
        }

        let frame_future = Some(Box::new(self.frame_future.take().unwrap().join(acquire_future)) as Box<_>);

        Some(Frame::new(self, frame_future, swapchain_image_index))
    }

    #[inline]
    fn create_framebuffers(
        dimensions: [u32; 2],
        swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
        attachments: &Attachments,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, (dimensions[1] as f32)],
            depth_range: 0.0..1.0,
        };

        dynamic_state.viewports = Some(vec![viewport]);

        swapchain_images
            .into_iter()
            .map(move |image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(attachments.diffuse.clone())
                        .unwrap()
                        .add(attachments.normals.clone())
                        .unwrap()
                        .add(attachments.light.clone())
                        .unwrap()
                        .add(attachments.depth.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<_>
            })
            .collect()
    }
}

pub struct Frame<'s> {
    system: &'s mut FrameSystem,
    frame_future: Option<Box<dyn GpuFuture>>,
    swapchain_image_index: usize,

    pass_index: u8,
    command_buffer: Option<AutoCommandBufferBuilder>,
}

impl<'s> Frame<'s> {
    fn new(
        system: &'s mut FrameSystem,
        frame_future: Option<Box<dyn GpuFuture>>,
        swapchain_image_index: usize,
    ) -> Self {
        Self {
            system,
            frame_future,
            swapchain_image_index,
            pass_index: 0,
            command_buffer: None,
        }
    }

    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 's>> {
        match {
            let pass_index = self.pass_index;
            self.pass_index += 1;
            pass_index
        } {
            0 => {
                self.command_buffer = Some(
                    AutoCommandBufferBuilder::primary_one_time_submit(
                        self.system.queue.device().clone(),
                        self.system.queue.family(),
                    )
                    .unwrap()
                    .begin_render_pass(
                        self.system.framebuffers[self.swapchain_image_index].clone(),
                        true,
                        vec![
                            [0.0, 0.0, 0.0, 0.0].into(),
                            [0.0, 0.0, 0.0, 0.0].into(),
                            [0.0, 0.0, 0.0, 0.0].into(),
                            [0.0, 0.0, 0.0, 0.0].into(),
                            (1.0, 0x00).into(),
                        ],
                    )
                    .unwrap(),
                );

                Some(Pass::Draw(DrawPass { frame: self }))
            }
            1 => {
                self.command_buffer = Some(self.command_buffer.take().unwrap().next_subpass(true).unwrap());
                Some(Pass::Compose(ComposingPass { frame: self }))
            }
            2 => {
                let command_buffer = self
                    .command_buffer
                    .take()
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap();

                let future = self
                    .frame_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.system.queue.clone(),
                        self.system.swapchain.clone(),
                        self.swapchain_image_index,
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        self.system.frame_future = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        self.system.invalidate_swapchain();
                        self.system.frame_future =
                            Some(Box::new(vulkano::sync::now(self.system.queue.device().clone())) as Box<_>);
                    }
                    Err(e) => {
                        log::error!("Failed to flush future: {:?}", e);
                        self.system.frame_future =
                            Some(Box::new(vulkano::sync::now(self.system.queue.device().clone())) as Box<_>);
                    }
                }

                None
            }
            _ => None,
        }
    }

    #[inline]
    fn execute_secondary_buffer<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        unsafe {
            self.command_buffer = Some(
                self.command_buffer
                    .take()
                    .unwrap()
                    .execute_commands(command_buffer)
                    .unwrap(),
            )
        }
    }
}

pub enum Pass<'f, 's: 'f> {
    Draw(DrawPass<'f, 's>),
    Compose(ComposingPass<'f, 's>),
}

pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        self.frame.execute_secondary_buffer(command_buffer);
    }

    #[inline]
    pub fn dynamic_state(&self) -> &DynamicState {
        &self.frame.system.dynamic_state
    }
}

pub struct ComposingPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> ComposingPass<'f, 's> {
    pub fn compose(&mut self) {
        let command_buffer = self
            .frame
            .system
            .composing_system
            .draw(&self.frame.system.dynamic_state);

        self.frame.execute_secondary_buffer(command_buffer);
    }
}

#[derive(Clone)]
struct Attachments {
    diffuse: Arc<AttachmentImage>,
    normals: Arc<AttachmentImage>,
    light: Arc<AttachmentImage>,
    depth: Arc<AttachmentImage>,
}

impl Attachments {
    fn new(device: Arc<Device>, dimensions: [u32; 2]) -> Self {
        let diffuse =
            AttachmentImage::transient_input_attachment(device.clone(), dimensions, Format::A2B10G10R10UnormPack32)
                .unwrap();

        let normals =
            AttachmentImage::transient_input_attachment(device.clone(), dimensions, Format::A2B10G10R10UnormPack32)
                .unwrap();

        let light =
            AttachmentImage::transient_input_attachment(device.clone(), dimensions, Format::A2B10G10R10UnormPack32)
                .unwrap();

        let depth = AttachmentImage::transient_input_attachment(device, dimensions, Format::D24Unorm_S8Uint).unwrap();

        Self {
            diffuse,
            normals,
            light,
            depth,
        }
    }
}

impl From<Attachments> for ComposingSystemInput {
    fn from(attachments: Attachments) -> Self {
        Self {
            diffuse: attachments.diffuse,
            light: attachments.light,
            depth: attachments.depth,
        }
    }
}
