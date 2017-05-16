use std::io::Cursor;

use cgmath::{Rad, PerspectiveFov, Angle, Matrix4};
use glium::backend::{Facade};
use glium::index::{NoIndices, PrimitiveType};
use glium::texture::{RawImage2d, SrgbTexture2d};
use glium::draw_parameters::{DepthTest, BackfaceCullingMode};
use glium::{Surface, VertexBuffer, Program, Depth, DrawParameters};
use image;

use world3d::{Camera, World};
use Frame;

#[derive(Copy, Clone)]
struct Vertex {
    v_position: [f32; 3],
    v_tex_coords: [f32; 2],
}

implement_vertex!(Vertex, v_position, v_tex_coords);

pub struct Renderer {
    program: Program,
    texture: SrgbTexture2d,
}

impl Renderer {
    pub fn init(context: &Facade) -> Self {
        // Create the shader program to render with
        let vertex_shader_src = include_str!("./shader_vert.glsl");
        let fragment_shader_src = include_str!("./shader_frag.glsl");
        let program = Program::from_source(
            context,
            vertex_shader_src, fragment_shader_src,
            None
        ).unwrap();

        // Create the texture to render
        let image = image::load(
            Cursor::new(&include_bytes!("./texture.png")[..]),
            image::PNG
        ).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(
            image.into_raw(), image_dimensions
        );
        let texture = SrgbTexture2d::new(context, image).unwrap();

        Renderer {
            program,
            texture,
        }
    }

    pub fn render(&self, context: &Facade, frame: &mut Frame, camera: &Camera, world: &World) {
        // Create the vertex buffer
        let mut vertices = Vec::new();
        vertices.push(Vertex {v_position: [0.0, 0.0, 0.0], v_tex_coords: [0.0, 0.0]});
        vertices.push(Vertex {v_position: [1.0, 0.0, 0.0], v_tex_coords: [1.0, 0.0]});
        vertices.push(Vertex {v_position: [0.0, 1.0, 0.0], v_tex_coords: [0.0, 1.0]});
        let vertex_buffer = VertexBuffer::new(context, &vertices).unwrap();
        let indices = NoIndices(PrimitiveType::TrianglesList);

        // Create the uniforms
        let perspective = PerspectiveFov {
            fovy: Rad::full_turn() * 0.25,
            aspect: frame.size.x as f32 / frame.size.y as f32,
            near: 0.1,
            far: 500.0,
        };
        let projection = Matrix4::from(perspective);
        let view = camera.create_world_to_view_matrix();

        // Set up the drawing parameters
        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        // Go over everything in the world
        for position in &world.positions {
            // Create a matrix for this world entity
            let model = Matrix4::from_translation(*position);
            let matrix_raw: [[f32; 4]; 4] = (projection * view * model).into();

            // Perform the actual draw
            let uniforms = uniform! { u_matrix: matrix_raw, u_texture: &self.texture };
            frame.inner.draw(
                &vertex_buffer, &indices,
                &self.program, &uniforms,
                &params,
            ).unwrap();
        }
    }
}
