use std::path::{PathBuf};

pub struct Texture {
    // Texture needs to track the data to load a texture internally because the backend may need to
    // be re-loaded, in which case the backend data gets purged.
    // TODO: Make fields that can be private private, improve encapsulation
    pub source: PathBuf,
    pub format: TextureFormat,
    pub submitted: bool,
}

impl Texture {
    pub fn new<P: Into<PathBuf>>(path: P, format: TextureFormat) -> Self {
        // TODO: Remove this note when implementation is done
        // Texture will be loaded on-demand or when told, but always on a separate thread. This
        //  allows cobalt to provide non-blocking texture loading. Sometimes this means that during
        //  rendering a model has to be skipped (or rendered without texture, provide choice!)
        //  because there's no texture loaded yet. The game using cobalt needs to be able to detect
        //  if the needed textures are loaded and show loading screens otherwise.
        // Create a "Texture Set" or something similar that can be easily used to check if textures
        //  are loaded and submit them for loading on backend switch. This may need to be some more
        //  generic type of asset set, perhaps including non-core assets in some way. This wouldn't
        //  be the required way to upload textures but just a way to pre-load a bunch at once, and
        //  detect if they're loaded.

        Texture {
            source: path.into(),
            format,
            submitted: false,
        }
    }
}

#[derive(PartialEq)]
pub enum TextureFormat {
    Srgb,
    Linear,
    LinearRed,
}
