use winit::dpi::PhysicalPosition;
use winit::event::*;

use super::device_input_state::*;

pub struct InputState {
    keyboard: InputStateBuffers<KeyboardState>,
    mouse: InputStateBuffers<MouseButtonsState>,
    mouse_position: MousePosition,
}

#[allow(dead_code)]
impl InputState {
    pub fn new() -> Self {
        Self {
            keyboard: InputStateBuffers::new(),
            mouse: InputStateBuffers::new(),
            mouse_position: MousePosition::new(),
        }
    }

    pub fn flush(&mut self, handler: &InputStateHandler) {
        self.keyboard.update(&handler.keyboard);
        self.mouse.update(&handler.mouse);
        self.mouse_position.update(&handler.mouse_position);
    }

    #[inline]
    pub fn keyboard(&self) -> &InputStateBuffers<KeyboardState> {
        &self.keyboard
    }

    #[inline]
    pub fn mouse(&self) -> &InputStateBuffers<MouseButtonsState> {
        &self.mouse
    }

    #[inline]
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

#[allow(dead_code)]
impl<T> InputStateBuffers<T>
where
    T: Clone + Default + DeviceInputState,
{
    fn new() -> Self {
        Self {
            current: Default::default(),
            previous: Default::default(),
            any_pressed: false,
            any_released: false,
            last_pressed_key: None,
        }
    }

    pub fn update(&mut self, handler: &InputStateBuffersHandler<T>) {
        self.previous.clone_from(&self.current);
        self.current.clone_from(&handler.state);
        self.any_pressed = handler.any_pressed;
        self.any_released = handler.any_released;
        self.last_pressed_key.clone_from(&handler.last_pressed_key);
    }

    #[inline]
    pub fn last_pressed_key(&self) -> Option<T::Key> {
        self.last_pressed_key
    }

    #[inline]
    pub fn is_pressed(&self, key: T::Key) -> bool {
        self.current.is_pressed(key)
    }

    #[inline]
    pub fn is_released(&self, key: T::Key) -> bool {
        self.current.is_pressed(key)
    }

    #[inline]
    pub fn was_pressed(&self, key: T::Key) -> bool {
        !self.previous.is_pressed(key) && self.current.is_pressed(key)
    }

    #[inline]
    pub fn was_released(&self, key: T::Key) -> bool {
        self.previous.is_pressed(key) && !self.current.is_pressed(key)
    }

    #[inline]
    pub fn was_any_pressed(&self) -> bool {
        self.any_pressed
    }

    #[inline]
    pub fn was_any_released(&self) -> bool {
        self.any_released
    }
}

pub struct MousePosition {
    current: PhysicalPosition<f64>,
    previous: PhysicalPosition<f64>,
}

impl MousePosition {
    pub fn new() -> Self {
        Self {
            current: PhysicalPosition::new(0.0, 0.0),
            previous: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn update(&mut self, handler: &MousePositionHandler) {
        self.previous = self.current;
        self.current = handler.state;
    }

    #[inline]
    pub fn current(&self) -> &PhysicalPosition<f64> {
        &self.current
    }

    #[inline]
    pub fn delta(&self) -> PhysicalPosition<f64> {
        PhysicalPosition::new(self.current.x - self.previous.x, self.current.y - self.previous.y)
    }
}

pub struct InputStateHandler {
    keyboard: InputStateBuffersHandler<KeyboardState>,
    mouse: InputStateBuffersHandler<MouseButtonsState>,
    mouse_position: MousePositionHandler,
}

impl InputStateHandler {
    pub fn new() -> Self {
        Self {
            keyboard: InputStateBuffersHandler::new(),
            mouse: InputStateBuffersHandler::new(),
            mouse_position: Default::default(),
        }
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

                self.keyboard.handle_key(*state, *key);
            }
            WindowEvent::MouseInput { button, state, .. } => self.mouse.handle_key(*state, *button),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position.handle_movement(position);
            }
            _ => {}
        }
    }
}

pub struct InputStateBuffersHandler<T>
where
    T: DeviceInputState,
{
    state: T,
    any_pressed: bool,
    any_released: bool,
    last_pressed_key: Option<T::Key>,
}

impl<T> InputStateBuffersHandler<T>
where
    T: DeviceInputState,
{
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            any_pressed: false,
            any_released: false,
            last_pressed_key: None,
        }
    }

    pub fn handle_key(&mut self, state: ElementState, key: T::Key) {
        match state {
            ElementState::Pressed => {
                if !self.state.is_pressed(key) {
                    self.any_pressed = true;
                }
                self.state.press(key);
                self.last_pressed_key = Some(key);
            }
            ElementState::Released => {
                if !self.state.is_pressed(key) {
                    self.any_released = true;
                }
                self.state.release(key)
            }
        }
    }

    pub fn flush(&mut self) {
        self.any_pressed = false;
        self.any_released = false;
        self.last_pressed_key = None;
    }
}

pub struct MousePositionHandler {
    state: PhysicalPosition<f64>,
}

impl MousePositionHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_movement(&mut self, new_position: &PhysicalPosition<f64>) {
        self.state = *new_position;
    }
}

impl Default for MousePositionHandler {
    fn default() -> Self {
        Self {
            state: PhysicalPosition::new(0.0, 0.0),
        }
    }
}
