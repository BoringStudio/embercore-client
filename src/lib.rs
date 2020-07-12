extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use std::path::Path;

use anyhow::Result;
use once_cell::sync::OnceCell;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use embercore::tme;

use crate::config::Config;
use crate::game::camera::Camera;
use crate::game::GameState;
use crate::input::{InputState, InputStateHandler};
use crate::rendering::*;
use std::sync::{Arc, Mutex};

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

    let game_state = Arc::new(Mutex::new(GameState::new()));

    std::thread::spawn({
        let game_state = game_state.clone();

        move || {
            let mut now = std::time::Instant::now();

            loop {
                let then = std::time::Instant::now();
                let dt = (then - now).as_secs_f32();
                now = then;

                {
                    game_state.lock().unwrap().step(dt);
                }

                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        }
    });

    std::thread::spawn({
        let device = device.clone();
        let queue = queue.clone();

        move || {
            let content_dir = Path::new("content");

            let map = match resources::load_json(&content_dir.join("tilemap.json")).unwrap() {
                tme::Map::Orthogonal(map) => map,
                _ => panic!("Unsupported map type"),
            };

            let tileset_first_gid = match map.tile_sets.first() {
                Some(tme::TilesetContainer::TilesetRef(tileset_ref)) => {
                    let tileset = resources::load_json::<tme::Tileset>(&content_dir.join(&tileset_ref.source)).unwrap();
                    let (texture_view, size) =
                        load_tileset(&device, &queue, &content_dir.join(&tileset.image.unwrap()));

                    let _ = tx.send(ResourcesEvent::TileSetLoaded { texture_view, size });

                    tileset_ref.first_gid
                }
                _ => panic!("No tilesets found"),
            };

            let chunks_in_column = (map.height as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;
            let chunks_in_row = (map.width as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

            let mut chunk = Vec::<u16>::new();
            chunk.resize(CHUNK_SIZE * CHUNK_SIZE, 0);

            let slice_offset_matrix_size = std::mem::size_of::<f32>() * 4 * 4;
            let slice_tiles_array_size = std::mem::size_of::<u16>() * chunk.len();

            let mut slice_buffer = Vec::<u8>::new();
            slice_buffer.resize(slice_offset_matrix_size + slice_tiles_array_size, 0);

            for layer in map.layers.iter().filter_map(|item| {
                if let tme::Layer::TileLayer(tile_layer) = item {
                    Some(tile_layer)
                } else {
                    None
                }
            }) {
                let tiles = layer.data.extract_tiles(layer.compression).unwrap();

                for chunk_y in (0..chunks_in_column).map(|i| i * CHUNK_SIZE) {
                    for chunk_x in (0..chunks_in_row).map(|i| i * CHUNK_SIZE) {
                        let offset = glm::translation(&glm::vec3(chunk_x as f32, chunk_y as f32, 0.0));
                        slice_buffer[..slice_offset_matrix_size]
                            .copy_from_slice(bytemuck::cast_slice(offset.as_slice()));

                        let _ = chunk.iter_mut().map(|item| *item = 0).count();

                        let max_y = std::cmp::min(chunk_y + CHUNK_SIZE, map.height as usize) - chunk_y;
                        let max_x = std::cmp::min(chunk_x + CHUNK_SIZE, map.width as usize) - chunk_x;

                        for y in 0..max_y {
                            for x in 0..max_x {
                                let tile_index = (chunk_y + y) * map.width as usize + (chunk_x + x);
                                chunk[y * CHUNK_SIZE + x] = (tiles[tile_index] + 1 - tileset_first_gid) as u16;
                            }
                        }
                        slice_buffer[slice_offset_matrix_size..].copy_from_slice(bytemuck::cast_slice(&chunk));

                        let buffer = device.create_buffer_with_data(
                            slice_buffer.as_slice(),
                            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                        );

                        let _ = tx.send(ResourcesEvent::ChunkLoaded { offset, buffer });
                    }
                }
            }
        }
    });

    //

    let mut camera = Camera::new(window.inner_size());
    rendering_state
        .tilemap_renderer()
        .update_camera(&device, &camera.view(), &camera.projection());

    let mut input_state_handler = InputStateHandler::new();

    let mut chunks = Vec::new();

    let mut now = std::time::Instant::now();

    events_loop.run(move |event, _, control_flow| match event {
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
                .update_camera(&device, &camera.view(), &camera.projection());
        }
        Event::WindowEvent { ref event, .. } => {
            input_state_handler.handle_window_event(event);
        }
        Event::RedrawEventsCleared => {
            while let Ok(resources_event) = rx.try_recv() {
                match resources_event {
                    ResourcesEvent::TileSetLoaded { texture_view, size } => {
                        rendering_state
                            .tilemap_renderer()
                            .update_tileset(&device, &texture_view, &size);
                    }
                    ResourcesEvent::ChunkLoaded { buffer, .. } => {
                        let bind_group = rendering_state
                            .tilemap_renderer()
                            .create_chunk_bind_group(&device, &buffer);

                        chunks.push((buffer, bind_group));
                    }
                }
            }

            {
                let mut game_state = game_state.lock().unwrap();
                game_state.update_input_state(&input_state_handler);

                if let Some(view) = game_state.read_main_camera_view() {
                    camera.set_view(view);
                    rendering_state
                        .tilemap_renderer()
                        .update_camera(&device, &camera.view(), &camera.projection());
                }
            }

            input_state_handler.flush();

            let (mut encoder, mut frame) = rendering_state.frame();

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

enum ResourcesEvent {
    TileSetLoaded {
        texture_view: wgpu::TextureView,
        size: [u32; 2],
    },
    ChunkLoaded {
        offset: glm::Mat4,
        buffer: wgpu::Buffer,
    },
}

const CHUNK_SIZE: usize = 16;
