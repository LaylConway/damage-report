use input::{Input, Button, MouseButton, Motion, Key};

pub struct InputManager {
    pub camera_move_button: bool,
    pub forward_button: bool,
    pub left_button: bool,
    pub backward_button: bool,
    pub right_button: bool,

    frame: FrameInput,
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            camera_move_button: false,
            forward_button: false,
            left_button: false,
            backward_button: false,
            right_button: false,

            frame: FrameInput::new(),
        }
    }

    pub fn frame(&self) -> &FrameInput {
        &self.frame
    }

    pub fn new_frame(&mut self) {
        self.frame = FrameInput::new();
    }

    pub fn handle_event(&mut self, event: &Input) {
        match *event {
            Input::Press(Button::Mouse(MouseButton::Right)) =>
                self.camera_move_button = true,
            Input::Release(Button::Mouse(MouseButton::Right)) =>
                self.camera_move_button = false,

            Input::Press(Button::Keyboard(Key::W)) =>
                self.forward_button = true,
            Input::Release(Button::Keyboard(Key::W)) =>
                self.forward_button = false,

            Input::Press(Button::Keyboard(Key::A)) =>
                self.left_button = true,
            Input::Release(Button::Keyboard(Key::A)) =>
                self.left_button = false,

            Input::Press(Button::Keyboard(Key::S)) =>
                self.backward_button = true,
            Input::Release(Button::Keyboard(Key::S)) =>
                self.backward_button = false,

            Input::Press(Button::Keyboard(Key::D)) =>
                self.right_button = true,
            Input::Release(Button::Keyboard(Key::D)) =>
                self.right_button = false,

            Input::Move(Motion::MouseRelative(x, y)) => {
                self.frame.mouse_x += x as f32;
                self.frame.mouse_y += y as f32;
            },
            _ => {}
        }
    }
}

pub struct FrameInput {
    pub mouse_x: f32,
    pub mouse_y: f32,
}

impl FrameInput {
    pub fn new() -> Self {
        FrameInput {
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }
}
