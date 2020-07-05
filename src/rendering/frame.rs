use super::{RenderingState, TileMapRenderer};

pub struct Frame<'s> {
    rendering_state: &'s mut RenderingState,
    frame_output: wgpu::SwapChainTexture,
    state: FrameState,
}

impl<'s> Frame<'s> {
    pub(super) fn new(rendering_state: &'s mut RenderingState, frame_output: wgpu::SwapChainTexture) -> Self {
        Self {
            rendering_state,
            frame_output,
            state: FrameState::Draw,
        }
    }

    pub fn next_pass<'r>(&'r mut self) -> Option<Pass<'r, 's>> {
        match self.state.increment() {
            FrameState::Draw => Some(Pass::World(DrawPass { frame: self })),
            _ => None,
        }
    }

    pub fn submit(self, encoder: wgpu::CommandEncoder) {
        self.rendering_state.queue().submit(Some(encoder.finish()))
    }
}

#[derive(Copy, Clone)]
enum FrameState {
    Draw,
    Compose,
    Submit,
    End,
}

impl FrameState {
    fn increment(&mut self) -> Self {
        std::mem::replace(
            self,
            match self {
                FrameState::Draw => FrameState::Submit, // FrameState::Compose,
                // FrameState::Compose => FrameState::Submit,
                _ => FrameState::End,
            },
        )
    }
}

pub enum Pass<'r, 's> {
    World(DrawPass<'r, 's>),
}

pub struct DrawPass<'r, 's> {
    frame: &'r mut Frame<'s>,
}

impl<'r, 's> DrawPass<'r, 's> {
    pub fn start<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.frame.frame_output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        })
    }

    pub fn tile_map_renderer(&self) -> &TileMapRenderer {
        &self.frame.rendering_state.tilemap_renderer
    }
}
