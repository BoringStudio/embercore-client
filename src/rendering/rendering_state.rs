use crate::rendering::prelude::*;

use super::error;
use super::systems::frame_system::*;
use super::systems::world_rendering_system::*;

pub struct RenderingState {
    frame_system: FrameSystem,
    world_rendering_system: WorldRenderingSystem,
}

impl RenderingState {
    pub fn new(instance: Arc<Instance>, surface: Arc<Surface<Window>>) -> Result<Self, error::Error> {
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .context(error::NoDeviceAvailable)?;

        let queue_family = physical
            .queue_families()
            .filter(|&family| family.supports_graphics() && surface.is_supported(family).unwrap_or(false))
            .fold(None, |result: Option<QueueFamily>, family| match result {
                Some(result) if family.queues_count() > result.queues_count() => Some(family),
                Some(_) => result,
                _ if family.queues_count() > 0 => Some(family),
                _ => None,
            })
            .context(error::NoSuitableQueuesFound {
                device: physical.name(),
            })?;

        let (_, mut queues) = Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                khr_swapchain: true,
                khr_maintenance1: true,
                ..DeviceExtensions::none()
            },
            [(queue_family, 0.5)].iter().cloned(),
        )
        .context(error::DeviceCreation)?;

        let main_queue = queues.next().unwrap(); // at least one queue exists

        let frame_system = FrameSystem::new(surface.clone(), main_queue.clone());

        let identity_view_data_source = IdentityViewDataSource;
        let world_rendering_system = WorldRenderingSystem::new(
            main_queue.clone(),
            frame_system.deferred_subpass(),
            &identity_view_data_source,
        );

        Ok(Self {
            frame_system,
            world_rendering_system,
        })
    }

    pub fn handle_resize(&mut self) {
        self.frame_system.invalidate_swapchain();
    }

    #[inline(always)]
    pub fn frame(&mut self) -> Option<Frame> {
        self.frame_system.frame()
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn world_rendering_system(&mut self) -> &mut WorldRenderingSystem {
        &mut self.world_rendering_system
    }
}
