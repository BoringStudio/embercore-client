use super::FrameSystem;
use crate::rendering::prelude::*;

pub struct Frame<'s> {
    system: &'s mut FrameSystem,
    frame_future: Option<Box<dyn GpuFuture>>,
    swapchain_image_index: usize,

    pass_index: u8,
    command_buffer: Option<AutoCommandBufferBuilder>,
}

impl<'s> Frame<'s> {
    pub(super) fn new(
        system: &'s mut FrameSystem,
        frame_future: Option<Box<dyn GpuFuture>>,
        swapchain_image_index: usize,
    ) -> Self {
        Self {
            system,
            frame_future,
            swapchain_image_index,
            pass_index: 0,
            command_buffer: None,
        }
    }

    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 's>> {
        match {
            let pass_index = self.pass_index;
            self.pass_index += 1;
            pass_index
        } {
            0 => {
                let mut command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                    self.system.queue.device().clone(),
                    self.system.queue.family(),
                )
                .unwrap();
                command_buffer
                    .begin_render_pass(
                        self.system.framebuffers[self.swapchain_image_index].clone(),
                        true,
                        vec![
                            [0.0, 0.0, 0.0, 0.0].into(),
                            [0.0, 0.0, 0.0, 0.0].into(),
                            (1.0, 0x00).into(),
                        ],
                    )
                    .unwrap();

                self.command_buffer = Some(command_buffer);
                Some(Pass::Draw(DrawPass { frame: self }))
            }
            1 => {
                let mut command_buffer = self.command_buffer.take().unwrap();
                command_buffer.next_subpass(true).unwrap();

                self.command_buffer = Some(command_buffer);
                Some(Pass::Compose(ComposingPass { frame: self }))
            }
            2 => {
                let mut command_buffer = self.command_buffer.take().unwrap();
                command_buffer.end_render_pass().unwrap();

                let command_buffer = command_buffer.build().unwrap();

                let future = self
                    .frame_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.system.queue.clone(),
                        self.system.swapchain.clone(),
                        self.swapchain_image_index,
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        self.system.frame_future = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        self.system.invalidate_swapchain();
                        self.system.frame_future =
                            Some(Box::new(vulkano::sync::now(self.system.queue.device().clone())) as Box<_>);
                    }
                    Err(e) => {
                        log::error!("Failed to flush future: {:?}", e);
                        self.system.frame_future =
                            Some(Box::new(vulkano::sync::now(self.system.queue.device().clone())) as Box<_>);
                    }
                }

                None
            }
            _ => None,
        }
    }

    #[inline]
    fn execute_secondary_buffer<C>(&mut self, secondary_command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        let mut command_buffer = self.command_buffer.take().unwrap();

        unsafe {
            command_buffer.execute_commands(secondary_command_buffer).unwrap();
        }

        self.command_buffer = Some(command_buffer);
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
