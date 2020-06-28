mod frame;

pub use self::frame::*;

use super::composing_subsystem::*;
use crate::rendering::prelude::*;
use crate::rendering::screen_quad::ScreenQuad;

pub struct FrameSubsystem {
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,

    swapchain: Arc<Swapchain<Window>>,
    attachments: Attachments,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    should_recreate_swapchain: bool,
    frame_future: Option<Box<dyn GpuFuture>>,

    composing_system: ComposingSubsystem,
}

impl FrameSubsystem {
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
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: ImageViewAccess::format(&attachments.depth),
                        samples: 1,
                    }
                },
                passes: [
                    {
                        color: [diffuse],
                        depth_stencil: {depth},
                        input: []
                    },
                    {
                        color: [final_color],
                        depth_stencil: {},
                        input: [diffuse]
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

        let composing_subpass = Subpass::from(render_pass.clone(), 1).unwrap();
        let composing_system = ComposingSubsystem::new(
            queue.clone(),
            composing_subpass,
            &screen_quad,
            attachments.clone().into(),
        );

        let frame_future = Some(vulkano::sync::now(queue.device().clone()).boxed());

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

        let frame_future = Some(self.frame_future.take().unwrap().join(acquire_future).boxed());

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
                        .add(image)
                        .unwrap()
                        .add(attachments.diffuse.clone())
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

#[derive(Clone)]
struct Attachments {
    diffuse: Arc<AttachmentImage>,
    depth: Arc<AttachmentImage>,
}

impl Attachments {
    fn new(device: Arc<Device>, dimensions: [u32; 2]) -> Self {
        let diffuse =
            AttachmentImage::transient_input_attachment(device.clone(), dimensions, Format::A2B10G10R10UnormPack32)
                .unwrap();

        let depth = AttachmentImage::transient_input_attachment(device, dimensions, Format::D24Unorm_S8Uint).unwrap();

        Self { diffuse, depth }
    }
}

impl From<Attachments> for ComposingSystemInput {
    fn from(attachments: Attachments) -> Self {
        Self {
            diffuse: attachments.diffuse,
            depth: attachments.depth,
        }
    }
}
