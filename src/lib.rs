extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use std::sync::Arc;

use anyhow::Result;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImageLayout, ImageUsage, ImmutableImage, MipmapsCount, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::sync::{Fence, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use embercore::tme;

use crate::config::Config;
use crate::input::InputState;
use crate::rendering::frame_system::Pass;
use crate::rendering::RenderingState;
use vulkano::command_buffer::submit::SubmitCommandBufferBuilder;

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
    let mut rendering_state = RenderingState::new(instance.clone(), surface.clone())?;

    //
    let fence = Arc::new(Fence::from_pool(rendering_state.device().clone().clone())?);
    let resources_queue = rendering_state.resources_queue();

    std::thread::spawn({
        let fence = fence.clone();
        let resources_queue = resources_queue.clone();
        move || {
            let tileset = resources::load_tileset("./content/tileset.json").unwrap();
            let tileset_source = tileset.image.expect("Tileset image not specified");

            let tileset_image = resources::load_texture("./content", &tileset_source).unwrap();

            println!("Started submitting");

            let (texture, texture_future) = {
                let dimensions = Dimensions::Dim2d {
                    width: tileset_image.width(),
                    height: tileset_image.height(),
                };

                ImmutableImage::from_iter(
                    tileset_image.iter().cloned(),
                    dimensions,
                    Format::R8G8B8A8Srgb,
                    resources_queue.clone(),
                )
                .expect("Unable to load tileset image")
            };

            unsafe {
                let mut builder = SubmitCommandBufferBuilder::new();
                builder.set_fence_signal(&fence);
                builder.submit(resources_queue.as_ref()).unwrap();
            }
        }
    });

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
            rendering_state.handle_resize();
        }
        Event::WindowEvent { ref event, .. } => {
            input_state.handle_window_event(event);
        }
        Event::RedrawEventsCleared => {
            let mut _frame = match rendering_state.frame() {
                Some(frame) => frame,
                None => return,
            };

            if fence.ready().unwrap() {
                println!("Ready!");
            }

            input_state.flush(); // TODO: maybe move into ecs?

            // TODO: fill frame using rendering system from ecs
            while let Some(pass) = _frame.next_pass().unwrap() {
                match pass {
                    Pass::Draw(_) => {}
                    Pass::Compose(mut pass) => pass.compose(),
                }
            }
        }
        _ => {}
    });
}
