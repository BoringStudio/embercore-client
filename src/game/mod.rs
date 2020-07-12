pub mod camera;

use std::sync::Arc;

use specs::prelude::*;

use self::camera::*;

#[derive(Default)]
struct DeltaTime(pub f32);

struct WindowSize(pub winit::dpi::PhysicalSize<u32>);

struct Position(pub glm::Vec3);

impl Component for Position {
    type Storage = VecStorage<Self>;
}

struct CameraComponent(pub Arc<Camera>);

struct CameraMovementSystem;

impl<'a> System<'a> for CameraMovementSystem {
    type SystemData = (Read<'a, DeltaTime>);

    fn run(&mut self, data: Self::SystemData) {}
}

pub struct GameState {
    world: World,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert(DeltaTime(0.0));

        world.register::<Position>();

        let mut camera_movement_system = CameraMovementSystem;
        camera_movement_system.run_now(&world);
        world.maintain();

        Self { world }
    }

    pub fn update_dt(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().0 = dt;
    }
}
