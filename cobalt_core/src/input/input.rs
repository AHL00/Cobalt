pub use winit::event::MouseButton;
use winit::event::WindowEvent;
pub use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

// TODO: Clean up and re-organise into multiple files

pub struct Input {
    keyboard: Keyboard,
    mouse: Mouse,
}

/// Represents changes in input state.
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyboardEvent(KeyboardEvent),
    MouseEvent(MouseEvent),
}

/// Represents changes in keyboard state.
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// The key was pressed. Only triggers once even if the key is held.
    Pressed(KeyCode),
    /// The key was released.
    Released(KeyCode),
}

/// Represents changes in mouse state.
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// The mouse button was pressed. Only triggers once even if the button is held.
    Pressed(MouseButton),
    /// The mouse button was released.
    Released(MouseButton),
    /// The mouse was moved by the given delta.
    Moved(f32, f32),
    /// The mouse wheel was scrolled by the given delta.
    Scrolled(f32, f32),
}

pub trait InputInternal {
    /// Called on every new event.
    /// Returns an `InputEvent` if the input state was changed.
    /// This also means that the event was consumed.
    /// Returns: (Event / State change, Consumed).
    /// Sometimes, the event is consumed but the input state wasn't changed.
    fn update(&mut self, event: &WindowEvent) -> (Option<InputEvent>, bool);

    /// Called between frames.
    /// This should be called after functions that use the input state.
    fn prepare(&mut self);

    fn new() -> Self;
}

/// To be used by other engine crates.
impl InputInternal for Input {
    fn update(&mut self, event: &WindowEvent) -> (Option<InputEvent>, bool) {
        let (keyboard_event, keyboard_consumed) = self.keyboard.update(event); 

        let (mouse_event, mouse_consumed) = self.mouse.update(event);

        let consumed = keyboard_consumed || mouse_consumed;

        let input_event = match (keyboard_event, mouse_event) {
            (Some(keyboard_event), None) => Some(InputEvent::KeyboardEvent(keyboard_event)),
            (None, Some(mouse_event)) => Some(InputEvent::MouseEvent(mouse_event)),
            _ => None,
        };

        (input_event, consumed)
    }

    fn prepare(&mut self) {
        self.keyboard.prepare();
        self.mouse.prepare();
    }

    fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }
}

impl Input {
    pub fn get_keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn get_mouse(&self) -> &Mouse {
        &self.mouse
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

impl ButtonState {
    pub fn is_pressed(&self) -> bool {
        match self {
            ButtonState::Pressed => true,
            ButtonState::Held { .. } => true,
            _ => false,
        }
    }

    pub fn is_held(&self) -> bool {
        match self {
            ButtonState::Held { .. } => true,
            _ => false,
        }
    }
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
    /// (Event, Consumed)
    pub(crate) fn update(&mut self, event: &WindowEvent) -> (Option<KeyboardEvent>, bool) {
        let key_event = match event {
            WindowEvent::KeyboardInput { event, .. } => Some(event),
            _ => None,
        };

        let mut key_found_in_vec = false;

        if let Some(key_event) = key_event {
            if key_event.repeat {
                return (None, true);
            }

            let physical_key = key_event.physical_key;

            // TODO: Fix, for now if the key is not a recognized key, it is ignored
            if let PhysicalKey::Code(key_code) = physical_key {
                let keyboard_event = match key_event.state {
                    winit::event::ElementState::Pressed => KeyboardEvent::Pressed(key_code),
                    winit::event::ElementState::Released => KeyboardEvent::Released(key_code),
                };

                for (key, state) in self.keys.iter_mut() {
                    if *key == key_code {
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

                // New key, which means it can't be held. It could be released if mouse
                // was held before start of app.
                if !key_found_in_vec {
                    let button_state = match key_event.state {
                        winit::event::ElementState::Pressed => ButtonState::Pressed,
                        winit::event::ElementState::Released => ButtonState::Released,
                    };

                    self.keys.push((key_code, button_state));
                }

                // Consumed and state changed
                (Some(keyboard_event), true)
            } else {
                // Consumed but state not changed as the key is not recognized
                (None, true)
            }
        } else {
            // Not consumed
            (None, false)
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
    /// Updates the keyboard state. 
    /// Called on every new event. 
    /// (Event, Consumed)
    pub(crate) fn update(&mut self, event: &WindowEvent) -> (Option<MouseEvent>, bool) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.delta = (
                    position.x as f32 - self.position.0,
                    position.y as f32 - self.position.1,
                );

                self.position = (position.x as f32, position.y as f32);

                (Some(MouseEvent::Moved(self.delta.0, self.delta.1)), true)
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // TODO: Mouse wheel input
                let _ = delta;

                (Some(MouseEvent::Scrolled(0.0, 0.0)), true)
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut button_found_in_vec = false;

                let mouse_event = match state {
                    winit::event::ElementState::Pressed => MouseEvent::Pressed(*button),
                    winit::event::ElementState::Released => MouseEvent::Released(*button),
                };

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

                (Some(mouse_event), true)
            }
            _ => (None, false),
        }
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
