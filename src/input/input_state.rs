use winit::dpi::PhysicalPosition;
use winit::event::*;

use super::device_input_state::*;

pub struct InputState {
    keyboard: InputStateBuffers<KeyboardState>,
    mouse: InputStateBuffers<MouseButtonsState>,
    mouse_position: MousePosition,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keyboard: InputStateBuffers::new(),
            mouse: InputStateBuffers::new(),
            mouse_position: MousePosition::new(),
        }
    }

    pub fn flush(&mut self) {
        self.keyboard.flush();
        self.mouse.flush();
        self.mouse_position.flush();
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode, state, ..
                },
                ..
            } => {
                let key = match virtual_keycode {
                    Some(key) => key,
                    None => return,
                };

                self.keyboard.handle_key(state, key);
            }
            WindowEvent::MouseInput { button, state, .. } => self.mouse.handle_key(state, button),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position.handle_movement(position);
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn keyboard(&self) -> &InputStateBuffers<KeyboardState> {
        &self.keyboard
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn mouse(&self) -> &InputStateBuffers<MouseButtonsState> {
        &self.mouse
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn mouse_position(&self) -> &MousePosition {
        &self.mouse_position
    }
}

pub struct InputStateBuffers<T>
where
    T: DeviceInputState,
{
    current: T,
    previous: T,
    any_pressed: bool,
    any_released: bool,
    last_pressed_key: Option<T::Key>,
}

impl<T> InputStateBuffers<T>
where
    T: Clone + Default + DeviceInputState,
    T::Key: Clone,
{
    fn new() -> Self {
        Default::default()
    }

    fn flush(&mut self) {
        self.previous.clone_from(&self.current);
        self.any_pressed = false;
        self.any_released = false;
        self.last_pressed_key = None;
    }

    fn handle_key(&mut self, state: &ElementState, key: &T::Key) {
        match state {
            ElementState::Pressed => {
                if !self.current.is_pressed(key) {
                    self.any_pressed = true;
                }
                self.current.press(key);
                self.last_pressed_key = Some(key.clone());
            }
            ElementState::Released => {
                if !self.current.is_pressed(key) {
                    self.any_released = true;
                }
                self.current.release(key)
            }
        }
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn last_pressed_key(&self) -> Option<T::Key> {
        self.last_pressed_key.clone()
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_pressed(&self, key: &T::Key) -> bool {
        self.current.is_pressed(key)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_released(&self, key: &T::Key) -> bool {
        self.current.is_pressed(key)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn was_pressed(&self, key: &T::Key) -> bool {
        !self.previous.is_pressed(key) && self.current.is_pressed(key)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn was_released(&self, key: &T::Key) -> bool {
        self.previous.is_pressed(key) && !self.current.is_pressed(key)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn was_any_pressed(&self) -> bool {
        self.any_pressed
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn was_any_released(&self) -> bool {
        self.any_released
    }
}

impl<T> Default for InputStateBuffers<T>
where
    T: Default + DeviceInputState,
{
    fn default() -> Self {
        Self {
            current: Default::default(),
            previous: Default::default(),
            any_pressed: false,
            any_released: false,
            last_pressed_key: None,
        }
    }
}

pub struct MousePosition {
    current: PhysicalPosition<f64>,
    previous: PhysicalPosition<f64>,
    initialized: bool,
}

impl MousePosition {
    pub fn new() -> Self {
        Self {
            current: PhysicalPosition::new(0.0, 0.0),
            previous: PhysicalPosition::new(0.0, 0.0),
            initialized: false,
        }
    }

    fn flush(&mut self) {
        self.previous = self.current;
    }

    fn handle_movement(&mut self, new_position: &PhysicalPosition<f64>) {
        self.current = *new_position;
        self.initialized = true;
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn current(&self) -> &PhysicalPosition<f64> {
        &self.current
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn delta(&self) -> PhysicalPosition<f64> {
        PhysicalPosition::new(self.current.x - self.previous.x, self.current.y - self.previous.y)
    }
}
