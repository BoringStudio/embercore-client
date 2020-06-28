extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use std::sync::Arc;

use anyhow::Result;
use png::ColorType;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::config::Config;
use crate::input::InputState;
use crate::rendering::*;

pub async fn run(_config: Config) -> Result<()> {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None)?
    };

    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1024, 768))
        .with_title("embercore")
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    //
    let mut rendering_state = RenderingState::new(instance, surface.clone())?;

    let mut camera = Camera::new(surface);
    camera.set_view(glm::identity());

    let mut tilemap_renderer = TileMapRenderer::new(
        rendering_state.main_queue().clone(),
        rendering_state.frame_system().deferred_subpass(),
        &camera,
    )?;

    let (sender, receiver) = std::sync::mpsc::channel();

    //
    std::thread::spawn({
        let resources_queue = rendering_state.resources_queue().clone();

        move || {
            let tileset = resources::load_tileset("./content/tileset.json").unwrap();
            let tileset_source = tileset.image.expect("Tileset image not specified");

            let (info, data) = resources::load_texture("./content", &tileset_source).unwrap();

            let (texture, texture_future) = {
                let dimensions = Dimensions::Dim2d {
                    width: info.width,
                    height: info.height,
                };

                let format = match info.color_type {
                    ColorType::RGB => Format::R8G8B8Srgb,
                    ColorType::RGBA => Format::R8G8B8A8Srgb,
                    _ => unreachable!(), // are not supported by `resources::load_texture`
                };

                ImmutableImage::from_iter(data.iter().cloned(), dimensions, format, resources_queue.clone())
                    .expect("Unable to load tileset image")
            };

            texture_future
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();

            let _ = sender.send(TileSetInfo {
                image: texture,
                size: [info.width as i32, info.height as i32],
            });
        }
    });

    let tile_mesh = TileMesh::new(rendering_state.main_queue().clone())?;
    let mesh_state = MeshState {
        transform: glm::translation(&glm::Vec3::new(0.0, 0.0, 0.0)),
    };

    let mut input_state = InputState::new();

    events_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            camera.update_projection();
            tilemap_renderer.update_view(&camera).unwrap();
            rendering_state.handle_resize();
        }
        Event::WindowEvent { ref event, .. } => {
            input_state.handle_window_event(event);
        }
        Event::RedrawEventsCleared => {
            let mut _frame = match rendering_state.frame().unwrap() {
                Some(frame) => frame,
                None => return,
            };

            input_state.flush(); // TODO: maybe move into ecs?

            if let Ok(image) = receiver.try_recv() {
                tilemap_renderer.update_tileset(image).unwrap();
            }

            // TODO: fill frame using rendering system from ecs
            while let Some(pass) = _frame.next_pass().unwrap() {
                match pass {
                    Pass::Draw(mut pass) => {
                        pass.execute(tilemap_renderer.draw(pass.dynamic_state(), &tile_mesh, &mesh_state));
                    }
                    Pass::Compose(mut pass) => pass.compose(),
                }
            }
        }
        _ => {}
    });
}

struct Camera {
    view: glm::Mat4,
    projection: glm::Mat4,
    scale: u32,

    surface: Arc<Surface<Window>>,
}

impl Camera {
    pub fn new(surface: Arc<Surface<Window>>) -> Self {
        let mut camera = Self {
            view: glm::identity(),
            projection: glm::identity(),
            surface,
            scale: 4,
        };
        camera.update_projection();
        camera
    }

    #[inline]
    pub fn set_view(&mut self, view: glm::Mat4) {
        self.view = view;
    }

    #[inline]
    pub fn update_projection(&mut self) {
        let size = self.surface.window().inner_size();
        let (width, height) = (size.width, size.height);
        let factor = 2.0 * self.scale as f32;

        self.projection = glm::ortho(
            -(width as f32 / factor),
            width as f32 / factor,
            -(height as f32 / factor),
            height as f32 / factor,
            -10.0,
            10.0,
        );
    }
}

impl ViewDataSource for Camera {
    fn view(&self) -> glm::Mat4 {
        self.view
    }

    fn projection(&self) -> glm::Mat4 {
        self.projection
    }
}
