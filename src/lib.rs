extern crate nalgebra_glm as glm;

pub mod config;
mod game;
mod input;
mod rendering;
mod resources;

use anyhow::Result;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImageUsage, ImmutableImage, SwapchainImage};
use vulkano::instance::Instance;
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
    let tileset = resources::load_tileset("./content/tileset.json")?;
    let tileset_source = tileset.image.expect("Tileset image not specified");

    let tileset_image = resources::load_texture("./content", &tileset_source)?;

    let (texture, texture_future) = {
        let dimensions = Dimensions::Dim2d {
            width: tileset_image.width(),
            height: tileset_image.height(),
        };

        ImmutableImage::from_iter(
            tileset_image.iter().cloned(),
            dimensions,
            Format::R8G8B8A8Srgb,
            rendering_state.main_queue().clone(),
        )
        .expect("Unable to load tileset image")
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

            input_state.flush(); // TODO: maybe move into ecs?

            // TODO: fill frame using rendering system from ecs
            while let Some(pass) = _frame.next_pass() {
                match pass {
                    Pass::Draw(_) => {}
                    Pass::Compose(mut pass) => pass.compose(),
                }
            }
        }
        _ => {}
    });
}
