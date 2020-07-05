use anyhow::Result;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use super::error::Error;
use super::frame::Frame;
use crate::rendering::TileMapRenderer;

pub struct RenderingState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    pub(super) tilemap_renderer: TileMapRenderer,
}

impl RenderingState {
    pub async fn new(window: &Window) -> Result<Self> {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::UnsafeFeatures::disallow(),
            )
            .await
            .ok_or_else(|| Error::NoSuitableAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .map_err(|_| Error::NoSuitableDevice)?;

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: SWAPCHAIN_FORMAT,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let tilemap_renderer = TileMapRenderer::new(&device, &queue);

        Ok(Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            tilemap_renderer,
        })
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;
    }

    pub fn frame(&mut self) -> (wgpu::CommandEncoder, Frame) {
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if let Ok(frame) = self.swap_chain.get_next_frame() {
            return (encoder, Frame::new(self, frame.output));
        }

        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);

        let frame = self
            .swap_chain
            .get_next_frame()
            .expect("Failed to acquire next swap chain texture");

        (encoder, Frame::new(self, frame.output))
    }

    #[inline]
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    #[inline]
    pub fn tilemap_renderer(&mut self) -> &mut TileMapRenderer {
        &mut self.tilemap_renderer
    }
}

pub const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
