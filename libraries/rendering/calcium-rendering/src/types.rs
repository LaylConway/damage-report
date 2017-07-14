use std::any::{Any};
use texture::{Texture};
use {WindowRenderer, Renderer};

/// An associated types container with all types for a backend.
pub trait Types: Any + Sized {
    type Renderer: Renderer + Any;
    type WindowRenderer: WindowRenderer<Self> + Any;
    type Frame: Any;

    type Texture: Texture<Self> + Any + Send + Sync;
}
