extern crate nalgebra_glm as glm;

pub mod config;
mod input;
mod rendering;

use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_serde::formats::SymmetricalBincode;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use vulkano::instance::Instance;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use embercore::*;

use crate::config::Config;
use crate::input::InputState;

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let intance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None)?
    };

    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(800, 600))
        .with_inner_size(LogicalSize::new(1024, 768))
        .with_title("embercore")
        .build_vk_surface(&events_loop, intance.clone())
        .unwrap();

    let mut input_state = InputState::new();

    events_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent { ref event, .. } => {
            input_state.handle_window_event(event);
        }
        Event::RedrawEventsCleared => {
            // TODO: draw
        }
        _ => {}
    });

    Ok(())
}
