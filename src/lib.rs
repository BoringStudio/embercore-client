extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use anyhow::Result;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::config::Config;
use crate::input::InputState;
use crate::rendering::*;

pub async fn run(_config: Config) -> Result<()> {
    let events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1024, 768))
        .with_title("embercore")
        .build(&events_loop)?;

    //
    let mut rendering_state: RenderingState = futures::executor::block_on(RenderingState::new(&window))?;

    //
    let texture_view = {
        let (texture_info, texture_data) = resources::load_texture("content", "tileset.png")?;

        let texture_extent = wgpu::Extent3d {
            width: texture_info.width,
            height: texture_info.height,
            depth: 1,
        };

        let texture = rendering_state.device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        rendering_state.queue().write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_data.as_slice(),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: texture_info.color_type.samples() as u32 * texture_info.width,
                rows_per_image: 0,
            },
            texture_extent,
        );

        texture.create_default_view()
    };

    //

    let mut camera = Camera::new(window.inner_size());
    camera.set_view(&glm::identity());

    let mut input_state = InputState::new();

    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                camera.update_projection(size);
                rendering_state.handle_resize(size);
            }
            Event::WindowEvent { ref event, .. } => {
                input_state.handle_window_event(event);
            }
            Event::RedrawEventsCleared => {
                let (mut encoder, mut frame) = rendering_state.frame();

                input_state.flush(); // TODO: maybe move into ecs?

                while let Some(pass) = frame.next_pass() {
                    match pass {
                        Pass::World(cx) => {
                            let mut pass = cx.start(&mut encoder);

                            let mut tilemap_renderer = cx.tile_map_renderer().start(&mut pass);
                            tilemap_renderer.draw_tile();
                        }
                    }
                }

                frame.submit(encoder);
            }
            _ => {}
        }
    })
}

struct Camera {
    view: glm::Mat4,
    projection: glm::Mat4,
    scale: u32,
}

impl Camera {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        let mut camera = Self {
            view: glm::identity(),
            projection: glm::identity(),
            scale: 2,
        };
        camera.update_projection(size);
        camera
    }

    #[inline]
    pub fn set_view(&mut self, view: &glm::Mat4) {
        self.view.copy_from(view);
    }

    #[inline]
    pub fn update_projection(&mut self, size: PhysicalSize<u32>) {
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
