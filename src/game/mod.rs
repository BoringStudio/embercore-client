pub mod camera;

use std::sync::Arc;

use specs::prelude::*;

use self::camera::*;
use crate::input::{InputState, InputStateHandler};
use specs::shred::Fetch;
use std::cell::Cell;
use std::ops::Deref;
use winit::event::VirtualKeyCode;

#[derive(Default)]
struct DeltaTime(pub f32);

#[derive(Default)]
pub struct MainCameraState {
    position: glm::Vec3,
    changed: bool,
}

impl MainCameraState {
    pub fn read_updated_view(&mut self) -> Option<glm::Mat4> {
        if self.changed {
            self.changed = false;
            Some(glm::translation(&self.position))
        } else {
            None
        }
    }
}

struct WindowSize(pub winit::dpi::PhysicalSize<u32>);

struct Position(pub glm::Vec3);

impl Component for Position {
    type Storage = VecStorage<Self>;
}

struct CameraComponent(pub Arc<Camera>);

struct CameraMovementSystem;

impl<'a> System<'a> for CameraMovementSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadExpect<'a, InputState>,
        Write<'a, MainCameraState>,
    );

    fn run(&mut self, (dt, input_state, mut camera_state): Self::SystemData) {
        let speed = 10.0;
        let mut direction = glm::vec3(0.0, 0.0, 0.0);
        let mut moved = false;
        if input_state.keyboard().is_pressed(VirtualKeyCode::D) {
            direction += glm::vec3(1.0, 0.0, 0.0);
            moved = true;
        } else if input_state.keyboard().is_pressed(VirtualKeyCode::A) {
            direction += glm::vec3(-1.0, 0.0, 0.0);
            moved = true;
        }
        if input_state.keyboard().is_pressed(VirtualKeyCode::W) {
            direction += glm::vec3(0.0, -1.0, 0.0);
            moved = true;
        } else if input_state.keyboard().is_pressed(VirtualKeyCode::S) {
            direction += glm::vec3(0.0, 1.0, 0.0);
            moved = true;
        }

        if moved {
            camera_state.position += direction * speed * dt.0;
            camera_state.changed = true;
        }
    }
}

pub struct GameState {
    world: World,
    camera_movement_system: CameraMovementSystem,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert(DeltaTime(0.0));
        world.insert(InputState::new());
        world.insert(MainCameraState::default());

        world.register::<Position>();

        let camera_movement_system = CameraMovementSystem;

        Self {
            world,
            camera_movement_system,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().0 = dt;

        self.camera_movement_system.run_now(&mut self.world);
        self.world.maintain();
    }

    pub fn update_input_state(&mut self, input_state_handler: &InputStateHandler) {
        self.world.write_resource::<InputState>().update(input_state_handler);
    }

    pub fn read_main_camera_view(&mut self) -> Option<glm::Mat4> {
        self.world.write_resource::<MainCameraState>().read_updated_view()
    }
}
