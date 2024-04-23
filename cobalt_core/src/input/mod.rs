pub mod input;
pub use input::*;

pub mod exports {
    pub use super::input::Input;
    pub use super::input::InputEvent;
    pub use super::input::KeyboardEvent;
    pub use super::input::MouseEvent;
    pub use super::input::ButtonState;
    pub use super::input::KeyCode;
    pub use super::input::MouseButton;
    pub use super::input::Keyboard;
    pub use super::input::Mouse;
}
