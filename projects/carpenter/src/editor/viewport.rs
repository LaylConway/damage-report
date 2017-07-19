use cgmath::{Vector2, Vector3, Quaternion, Rad, Zero, Euler, Angle};
use window::{AdvancedWindow};
use bus::{BusReader};

use calcium_rendering::{Error, Types, Texture, TextureFormat, Viewport, WindowRenderer};
use calcium_rendering_world3d::{RenderWorld, Camera, World3DRenderer, Entity, World3DTypes, Model, Material, World3DRenderTarget};

use model::{Application, ApplicationEvent};
use input_manager::{InputManager};

pub struct EditorViewport<T: Types, WT: World3DTypes<T>> {
    render_world: RenderWorld<T, WT>,
    events: BusReader<ApplicationEvent>,

    model: Model<T, WT>,
    material: Material<T>,

    camera_position: Vector3<f32>,
    camera_pitch: f32,
    camera_yaw: f32,
}

impl<T: Types, WT: World3DTypes<T>> EditorViewport<T, WT> {
    pub fn new(renderer: &mut T::Renderer, app: &mut Application) -> Result<Self, Error> {
        let mut render_world = RenderWorld::new();

        render_world.ambient_light = Vector3::new(0.05, 0.05, 0.05);
        render_world.directional_light = Vector3::new(1.0, 1.0, 1.0);

        let model = Model::<T, WT>::load(renderer, "./assets/cube.obj", 1.0);
        let material = Material {
            base_color: Texture::from_file(
                renderer, "./assets/texture.png", TextureFormat::Srgb
            )?,
            normal_map: Texture::from_file(
                renderer, "./assets/texture_normal.png", TextureFormat::Linear
            )?,
            metallic_map: Texture::from_file(
                renderer, "./assets/texture_metallic.png", TextureFormat::LinearRed
            )?,
            roughness_map: Texture::from_file(
                renderer, "./assets/texture_roughness.png", TextureFormat::LinearRed
            )?,
        };

        Ok(EditorViewport {
            render_world,
            events: app.subscribe(),

            model,
            material,

            camera_position: Vector3::new(0.0, 2.0, 5.0),
            camera_pitch: 0.0,
            camera_yaw: 0.0,
        })
    }

    pub fn update<W: AdvancedWindow>(
        &mut self, delta: f32, input: &InputManager, window: &mut W
    ) {
        // Check if we got events
        while let Ok(ev) = self.events.try_recv() {
            match ev {
                ApplicationEvent::NewBrush =>
                    self.add_brush(),
            }
        }

        // Update the camera based on input
        self.update_camera(delta, input, window);
    }

    pub fn render(
        &self,
        frame: &mut T::Frame,
        renderer: &mut T::Renderer,
        window_renderer: &mut T::WindowRenderer,
        world3d_renderer: &mut WT::Renderer,
        world3d_rendertarget: &mut World3DRenderTarget<T, WT>,
    ) {
        // Create a viewport that doesn't overlap the UI
        let viewport = Viewport::new(
            Vector2::new(0.0, 96.0),
            window_renderer.size().cast() - Vector2::new(0.0, 96.0),
        );

        world3d_renderer.render(
            &self.render_world, &self.create_camera(),
            world3d_rendertarget, &viewport,
            renderer, window_renderer, frame
        );
    }

    fn update_camera<W: AdvancedWindow>(
        &mut self, _delta: f32, input: &InputManager, window: &mut W
    ) {
        if !input.navigate_button() {
            window.set_capture_cursor(false);

            // We don't need to do anything more
            return;
        }

        window.set_capture_cursor(true);

        // Rotate the player's yaw depending on input
        let frame_input = input.frame();
        self.camera_yaw += frame_input.mouse_x * -0.0001;
        self.camera_pitch += frame_input.mouse_y * -0.0001;

        // Limit the pitch
        if self.camera_pitch > 0.25 {
            self.camera_pitch = 0.25;
        }
        if self.camera_pitch < -0.25 {
            self.camera_pitch = -0.25;
        }
    }

    fn add_brush(&mut self) {
        self.render_world.add_entity(Entity {
            position: Vector3::new(0.0, 0.0, 0.0),
            mesh: self.model.meshes[0].clone(),
            material: self.material.clone(),
        });
    }


    pub fn create_camera(&self) -> Camera {
        Camera {
            position: self.camera_position,
            rotation: self.create_camera_rotation(),
        }
    }

    fn create_camera_rotation(&self) -> Quaternion<f32> {
        let yaw: Quaternion<f32> =
            Euler::new(Rad::zero(), Rad::full_turn() * self.camera_yaw, Rad::zero()).into();
        let pitch: Quaternion<f32> =
            Euler::new(Rad::full_turn() * self.camera_pitch, Rad::zero(), Rad::zero()).into();
        yaw * pitch
    }
}
