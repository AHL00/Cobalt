pub use winit::event::MouseButton;
use winit::event::WindowEvent;
pub use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }

    /// Called on every new event.
    pub(crate) fn update(&mut self, event: &WindowEvent) {
        self.keyboard.update(event);
        self.mouse.update(event);
    }

    /// Called between frames.
    /// This should be called after functions that use the input state.
    pub(crate) fn prepare(&mut self) {
        self.keyboard.prepare();
        self.mouse.prepare();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonState {
    /// The button was pressed this frame.
    Pressed,
    /// The button has been held for multiple frames.
    Held {
        /// How long the button has been held.
        duration: std::time::Duration,
    },
    /// The button was released this frame.
    Released,
    /// The button is not pressed.
    NotPressed,
}

pub struct Keyboard {
    /// When a new key is pressed, it is added to this list.
    /// When a key is released, it is not removed from this list.
    /// Rather, it is marked as released.
    /// If a key isn't in this list, it is not pressed.
    pub(crate) keys: Vec<(KeyCode, ButtonState)>,
    last_prep: std::time::Instant,
}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Self {
            keys: Vec::new(),
            last_prep: std::time::Instant::now(),
        }
    }

    /// Prepares to update the keyboard state.
    /// Called before every new frame.
    pub(crate) fn prepare(&mut self) {
        let delta = std::time::Instant::now().duration_since(self.last_prep);

        for (_, state) in self.keys.iter_mut() {
            match state {
                ButtonState::Pressed => {
                    *state = ButtonState::Held {
                        duration: std::time::Duration::new(0, 0),
                    };
                }
                ButtonState::Released => {
                    *state = ButtonState::NotPressed;
                }
                ButtonState::Held { duration } => {
                    *duration += delta;
                }
                _ => (),
            }
        }

        self.last_prep = std::time::Instant::now();
    }

    /// Updates the keyboard state.
    /// Called on every new event.
    pub(crate) fn update(&mut self, event: &WindowEvent) {
        let key_event = match event {
            WindowEvent::KeyboardInput { event, .. } => Some(event),
            _ => None,
        };

        let mut key_found_in_vec = false;

        if let Some(key_event) = key_event {
            if key_event.repeat {
                return;
            }

            let physical_key = key_event.physical_key;

            match physical_key {
                PhysicalKey::Code(current_key) => {
                    for (key, state) in self.keys.iter_mut() {
                        if *key == current_key {
                            key_found_in_vec = true;

                            match key_event.state {
                                winit::event::ElementState::Pressed => {
                                    *state = ButtonState::Pressed;
                                }
                                winit::event::ElementState::Released => {
                                    *state = ButtonState::Released;
                                }
                            }
                        }
                    }

                    // New key, which means it can't be held or released
                    if !key_found_in_vec {
                        let button_state = match key_event.state {
                            winit::event::ElementState::Pressed => ButtonState::Pressed,
                            winit::event::ElementState::Released => ButtonState::Released,
                        };

                        self.keys.push((current_key, button_state));
                    }
                }
                _ => (),
            }
        }
    }

    pub fn get_key_state(&self, key: KeyCode) -> &ButtonState {
        for (k, state) in self.keys.iter() {
            if *k == key {
                return state;
            }
        }

        &ButtonState::NotPressed
    }
}

pub struct Mouse {
    // TODO: Side buttons
    pub(crate) buttons: Vec<(MouseButton, ButtonState)>,
    pub(crate) position: (f32, f32),
    pub(crate) delta: (f32, f32),
    last_prep: std::time::Instant,
}

impl Mouse {
    pub(crate) fn new() -> Self {
        Self {
            buttons: Vec::new(),
            position: (0.0, 0.0),
            delta: (0.0, 0.0),
            last_prep: std::time::Instant::now(),
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.delta = (0.0, 0.0);

        let time_delta = std::time::Instant::now().duration_since(self.last_prep);

        for (_, state) in self.buttons.iter_mut() {
            match state {
                ButtonState::Pressed => {
                    *state = ButtonState::Held {
                        duration: std::time::Duration::new(0, 0),
                    };
                }
                ButtonState::Released => {
                    *state = ButtonState::NotPressed;
                }
                ButtonState::Held { duration } => {
                    *duration += time_delta;
                }
                _ => (),
            }
        }

        self.last_prep = std::time::Instant::now();
    }

    pub(crate) fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.delta = (
                    position.x as f32 - self.position.0,
                    position.y as f32 - self.position.1,
                );

                self.position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // TODO: Mouse wheel
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut button_found_in_vec = false;

                for (b, s) in self.buttons.iter_mut() {
                    if *b == *button {
                        button_found_in_vec = true;

                        match state {
                            winit::event::ElementState::Pressed => {
                                *s = ButtonState::Pressed;
                            }
                            winit::event::ElementState::Released => {
                                *s = ButtonState::Released;
                            }
                        }
                    }
                }

                if !button_found_in_vec {
                    let button_state = match state {
                        winit::event::ElementState::Pressed => ButtonState::Pressed,
                        winit::event::ElementState::Released => ButtonState::Released,
                    };

                    self.buttons.push((*button, button_state));
                }
            }
            _ => (),
        };
    }

    pub fn get_button_state(&self, button: MouseButton) -> &ButtonState {
        for (b, state) in self.buttons.iter() {
            if *b == button {
                return state;
            }
        }

        &ButtonState::NotPressed
    }

    pub fn get_position(&self) -> (f32, f32) {
        self.position
    }

    pub fn get_delta(&self) -> (f32, f32) {
        self.delta
    }
}
