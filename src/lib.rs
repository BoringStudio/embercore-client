extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use std::path::Path;

use anyhow::Result;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use embercore::tme;

use crate::config::Config;
use crate::input::InputState;
use crate::rendering::*;
use wgpu::TextureView;

pub async fn run(_config: Config) -> Result<()> {
    let events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1024, 768))
        .with_title("embercore")
        .build(&events_loop)?;

    //
    let mut rendering_state: RenderingState = futures::executor::block_on(RenderingState::new(&window))?;
    let device = rendering_state.device().clone();
    let queue = rendering_state.queue().clone();

    //
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn({
        let device = device.clone();
        let queue = queue.clone();

        move || {
            let content_dir = Path::new("content");

            let map = match resources::load_json(&content_dir.join("tilemap.json")).unwrap() {
                tme::Map::Orthogonal(map) => map,
                _ => panic!("Unsupported map type"),
            };

            let (tileset_first_gid, tileset) = match map.tile_sets.first() {
                Some(tme::TilesetContainer::TilesetRef(tileset_ref)) => {
                    let tileset = resources::load_json::<tme::Tileset>(&content_dir.join(&tileset_ref.source)).unwrap();
                    (
                        tileset_ref.first_gid,
                        load_tileset(&device, &queue, &content_dir.join(&tileset.image.unwrap())),
                    )
                }
                _ => panic!("No tilesets found"),
            };
            let _ = tx.send(tileset.into());

            let tiles = match map.layers.first() {
                Some(tme::Layer::TileLayer(layer)) => layer.data.get_tiles(layer.compression).unwrap(),
                _ => panic!("No tile layers found"),
            };

            let mut chunk = Vec::with_capacity(16 * 16);
            for y in 0..16 {
                for x in 0..16 {
                    chunk.push((tiles[y * 16 as usize + x]) as u16);
                }
            }

            let chunk_uniform_buffer = device.create_buffer_with_data(
                bytemuck::cast_slice(&chunk),
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            );

            let _ = tx.send(ResourcesEvent::ChunkLoaded(chunk_uniform_buffer));
        }
    });

    //

    let mut camera = Camera::new(window.inner_size());
    camera.set_view(&(glm::scaling(&glm::vec3(32.0, 32.0, 1.0)) * glm::translation(&glm::vec3(-8.0, -8.0, 0.0))));
    rendering_state
        .tilemap_renderer()
        .update_camera(&device, &camera.view, &camera.projection);

    let mut input_state = InputState::new();

    let mut chunks = Vec::new();

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
                rendering_state
                    .tilemap_renderer()
                    .update_camera(&device, &camera.view, &camera.projection);
            }
            Event::WindowEvent { ref event, .. } => {
                input_state.handle_window_event(event);
            }
            Event::RedrawEventsCleared => {
                while let Ok(resources_event) = rx.try_recv() {
                    match resources_event {
                        ResourcesEvent::TileSetLoaded(texture_view, size) => {
                            rendering_state
                                .tilemap_renderer()
                                .update_tileset(&device, &texture_view, &size);
                        }
                        ResourcesEvent::ChunkLoaded(buffer) => {
                            let bind_group = rendering_state
                                .tilemap_renderer()
                                .create_chunk_bind_group(&device, &buffer);

                            chunks.push((buffer, bind_group));
                        }
                    }
                }

                let (mut encoder, mut frame) = rendering_state.frame();

                input_state.flush(); // TODO: maybe move into ecs?

                while let Some(pass) = frame.next_pass() {
                    match pass {
                        Pass::World(cx) => {
                            let mut pass = cx.start(&mut encoder);

                            let mut tilemap_renderer = cx.tile_map_renderer().start(&mut pass);
                            for (_, chunk) in chunks.iter() {
                                tilemap_renderer.draw_chunk(chunk);
                            }
                        }
                    }
                }

                frame.submit(encoder);
            }
            _ => {}
        }
    })
}

fn load_tileset(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &std::path::PathBuf,
) -> (wgpu::TextureView, [u32; 2]) {
    let (texture_info, texture_data) = resources::load_texture(path).unwrap();

    let (texture, texture_extent) =
        rendering::utils::create_rgba_texture(device, texture_info.width, texture_info.height);

    queue.write_texture(
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

    (texture.create_default_view(), [texture_info.width, texture_info.height])
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
            scale: 1,
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

enum ResourcesEvent {
    TileSetLoaded(wgpu::TextureView, [u32; 2]),
    ChunkLoaded(wgpu::Buffer),
}

impl From<(wgpu::TextureView, [u32; 2])> for ResourcesEvent {
    fn from((texture_view, size): (TextureView, [u32; 2])) -> Self {
        ResourcesEvent::TileSetLoaded(texture_view, size)
    }
}
