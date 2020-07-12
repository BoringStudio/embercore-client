use bit_set::BitSet;
use winit::event::{MouseButton, VirtualKeyCode};

pub trait DeviceInputState: Default {
    type Key: Copy;

    fn press(&mut self, key: Self::Key);
    fn release(&mut self, key: Self::Key);
    fn is_pressed(&self, key: Self::Key) -> bool;
}

#[derive(Clone)]
pub struct KeyboardState {
    keys: BitSet,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            keys: BitSet::with_capacity(256),
        }
    }
}

impl DeviceInputState for KeyboardState {
    type Key = VirtualKeyCode;

    #[inline]
    fn press(&mut self, key: Self::Key) {
        self.keys.insert(key as usize);
    }

    #[inline]
    fn release(&mut self, key: Self::Key) {
        self.keys.remove(key as usize);
    }

    #[inline]
    fn is_pressed(&self, key: Self::Key) -> bool {
        self.keys.contains(key as usize)
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct MouseButtonsState {
    buttons: BitSet,
}

impl MouseButtonsState {
    pub fn new() -> Self {
        Self {
            buttons: BitSet::with_capacity(32),
        }
    }

    #[inline(always)]
    fn get_index(button: MouseButton) -> usize {
        match button {
            MouseButton::Left => 0usize,
            MouseButton::Right => 1usize,
            MouseButton::Middle => 2usize,
            MouseButton::Other(other) => 3usize + other as usize,
        }
    }
}

impl DeviceInputState for MouseButtonsState {
    type Key = MouseButton;

    #[inline]
    fn press(&mut self, button: Self::Key) {
        self.buttons.insert(Self::get_index(button));
    }

    #[inline]
    fn release(&mut self, button: Self::Key) {
        self.buttons.remove(Self::get_index(button));
    }

    #[inline]
    fn is_pressed(&self, button: Self::Key) -> bool {
        self.buttons.contains(Self::get_index(button))
    }
}

impl Default for MouseButtonsState {
    fn default() -> Self {
        Self::new()
    }
}
