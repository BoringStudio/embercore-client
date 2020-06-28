use anyhow::Result;

use super::FrameSubsystem;
use crate::rendering::prelude::*;

pub struct Frame<'s> {
    system: &'s mut FrameSubsystem,
    frame_future: Option<Box<dyn GpuFuture>>,
    swapchain_image_index: usize,

    state: FrameState,
    command_buffer_builder: Option<AutoCommandBufferBuilder>,
}

impl<'s> Frame<'s> {
    pub(super) fn new(
        system: &'s mut FrameSubsystem,
        frame_future: Option<Box<dyn GpuFuture>>,
        swapchain_image_index: usize,
    ) -> Self {
        Self {
            system,
            frame_future,
            swapchain_image_index,
            state: FrameState::Draw,
            command_buffer_builder: None,
        }
    }

    pub fn next_pass<'f>(&'f mut self) -> Result<Option<Pass<'f, 's>>> {
        match self.state.increment() {
            FrameState::Draw => {
                let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
                    self.system.queue.device().clone(),
                    self.system.queue.family(),
                )?;

                command_buffer_builder.begin_render_pass(
                    self.system.framebuffers[self.swapchain_image_index].clone(),
                    true,
                    vec![
                        [0.0, 0.0, 0.0, 0.0].into(),
                        [0.0, 0.0, 0.0, 0.0].into(),
                        (1.0, 0x00).into(),
                    ],
                )?;

                self.command_buffer_builder = Some(command_buffer_builder);
                Ok(Some(Pass::Draw(DrawPass { frame: self })))
            }
            FrameState::Compose => {
                let mut command_buffer_builder = self.command_buffer_builder.take().unwrap();
                command_buffer_builder.next_subpass(true)?;

                self.command_buffer_builder = Some(command_buffer_builder);
                Ok(Some(Pass::Compose(ComposingPass { frame: self })))
            }
            FrameState::Submit => {
                let command_buffer = {
                    let mut command_buffer = self.command_buffer_builder.take().unwrap();
                    command_buffer.end_render_pass()?;
                    command_buffer.build()?
                };

                let future = self
                    .frame_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.queue.clone(), command_buffer)?
                    .then_swapchain_present(
                        self.system.queue.clone(),
                        self.system.swapchain.clone(),
                        self.swapchain_image_index,
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        self.system.frame_future = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        self.system.invalidate_swapchain();
                        self.system.frame_future = Some(vulkano::sync::now(self.system.queue.device().clone()).boxed());
                    }
                    Err(e) => {
                        log::error!("Failed to flush future: {:?}", e);
                        self.system.frame_future = Some(vulkano::sync::now(self.system.queue.device().clone()).boxed());
                    }
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    #[inline]
    fn execute_secondary_buffer<C>(&mut self, secondary_command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        let mut command_buffer = self.command_buffer_builder.take().unwrap();

        unsafe {
            command_buffer.execute_commands(secondary_command_buffer).unwrap();
        }

        self.command_buffer_builder = Some(command_buffer);
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
                FrameState::Draw => FrameState::Compose,
                FrameState::Compose => FrameState::Submit,
                _ => FrameState::End,
            },
        )
    }
}

pub enum Pass<'f, 's: 'f> {
    Draw(DrawPass<'f, 's>),
    Compose(ComposingPass<'f, 's>),
}

pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    #[allow(dead_code)]
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        self.frame.execute_secondary_buffer(command_buffer);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn dynamic_state(&self) -> &DynamicState {
        &self.frame.system.dynamic_state
    }
}

pub struct ComposingPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> ComposingPass<'f, 's> {
    pub fn compose(&mut self) {
        let command_buffer = self
            .frame
            .system
            .composing_system
            .draw(&self.frame.system.dynamic_state);

        self.frame.execute_secondary_buffer(command_buffer);
    }
}
