use crate::rendering::prelude::*;

use super::error::Error;
use super::systems::frame_system::*;
use super::systems::world_rendering_system::*;

pub struct RenderingState {
    device: Arc<Device>,
    main_queue: Arc<Queue>,
    resources_queue: Arc<Queue>,

    frame_system: FrameSystem,
    world_rendering_system: WorldRenderingSystem,
}

impl RenderingState {
    pub fn new(instance: Arc<Instance>, surface: Arc<Surface<Window>>) -> Result<Self> {
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or(Error::NoDeviceAvailable)?;

        let queue_family = physical
            .queue_families()
            .filter(|&family| family.supports_graphics() && surface.is_supported(family).unwrap_or(false))
            .fold(None, |result: Option<QueueFamily>, family| match result {
                Some(result) if family.queues_count() > result.queues_count() => Some(family),
                Some(_) => result,
                _ if family.queues_count() > 0 => Some(family),
                _ => None,
            })
            .ok_or(Error::NoSuitableQueuesFound {
                device: physical.name().to_owned(),
            })?;

        let (device, mut queues) = Device::new(
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
        .map_err(Error::DeviceCreation)?;

        let main_queue = queues.next().unwrap(); // at least one queue exists
        let resources_queue = queues.next().unwrap_or(main_queue.clone());

        let frame_system = FrameSystem::new(surface.clone(), main_queue.clone());

        let world_rendering_system = WorldRenderingSystem::new(
            main_queue.clone(),
            frame_system.deferred_subpass(),
            &IdentityViewDataSource,
        );

        Ok(Self {
            device,
            main_queue,
            resources_queue,
            frame_system,
            world_rendering_system,
        })
    }

    pub fn handle_resize(&mut self) {
        self.frame_system.invalidate_swapchain();
    }

    #[inline(always)]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    #[inline(always)]
    pub fn main_queue(&self) -> &Arc<Queue> {
        &self.main_queue
    }

    #[inline(always)]
    pub fn resources_queue(&self) -> &Arc<Queue> {
        &self.resources_queue
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
