use cgmath::{Vector2, Zero};
use calcium_rendering_simple2d::{Rectangle};
use style::{Style};

pub struct Element {
    pub style: Style,
    pub(crate) text: Option<ElementText>,

    // Cache data
    pub(crate) inner_id: i32,
    pub(crate) positioning: Positioning,

    // Input state
    pub(crate) cursor_state: ElementCursorState,
    pub(crate) clicked: bool,
}

impl Element {
    pub fn new(style: Style) -> Self {
        Element {
            inner_id: -1,

            style,
            text: None,

            positioning: Positioning::new(),

            cursor_state: ElementCursorState::None,
            clicked: false,
        }
    }

    pub fn with_text<S: Into<String>>(text: S, style: Style) -> Self {
        let mut element = Self::new(style);
        element.set_text(text);
        element
    }

    pub fn hovering(&self) -> bool {
        self.cursor_state == ElementCursorState::Hovering
    }

    pub fn held(&self) -> bool {
        self.cursor_state == ElementCursorState::Held
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        let text = text.into();

        if let Some(ref mut element_text) = self.text {
            element_text.set_text(text);
            return;
        }

        self.text = Some(ElementText::new(text));
    }
}

#[derive(Debug)]
pub struct Positioning {
    pub rectangle: Rectangle<f32>,
}

impl Positioning {
    pub fn new() -> Self {
        Positioning {
            rectangle: Rectangle::new(Vector2::zero(), Vector2::zero()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ElementCursorState {
    None,
    Hovering,
    Held,
}

#[derive(Debug)]
pub struct ElementText {
    pub(crate) text: String,
    pub(crate) cache_stale: bool,
    pub(crate) cache_rect: Rectangle<f32>,
}

impl ElementText {
    pub fn new(text: String) -> Self {
        ElementText {
            text: text,
            cache_stale: true,
            cache_rect: Rectangle::new(Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0)),
        }
    }

    pub fn set_text(&mut self, text: String) {
        if text != self.text {
            self.text = text;
            self.cache_stale = true;
        }
    }
}
