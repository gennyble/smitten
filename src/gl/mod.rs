mod rectangle;
mod texture;
mod transform;

pub use rectangle::Rectangle;
pub use texture::Texture;
pub use transform::Transform;

use std::{
    cell::{Cell, RefCell},
    path::Path as FilePath,
    rc::Rc,
};

use glow::{HasContext, Program};
use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

use crate::{Color, Vec2};

pub struct OpenGl {
    gl: Rc<glow::Context>,
    transform: Transform,
    program: Program,
    sdf: Program,
    clear_color: Color,
    draw_rect: Rectangle,
    bound_program: Cell<Program>,
}

impl OpenGl {
    pub fn new(context: &ContextWrapper<PossiblyCurrent, Window>, transform: Transform) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _)
        };

        // dirty hack to allow skip on this. Without let _ it would be an expression, and rustfmt::skip
        // on expressions is nightly
        #[rustfmt::skip]
        let _ = unsafe {
            // Repeat the texture if the coords go outside [0,1]
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);

            // Scale textures (mipmap?) using nearest neighbour which is better for pixely things
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            // Tell transparency do work how we'd expect.
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        };

        let program = unsafe {
            Self::create_program(
                &gl,
                include_str!("../../shaders/texture.vert"),
                include_str!("../../shaders/texture.frag"),
            )
        };

        let sdf = unsafe {
            Self::create_program(
                &gl,
                include_str!("../../shaders/sdf.vert"),
                include_str!("../../shaders/sdf.frag"),
            )
        };

        unsafe {
            gl.use_program(Some(program));
        }

        let draw_rect = Rectangle::new(&gl, (2.0, 2.0).into());

        Self {
            gl: Rc::new(gl),
            transform,
            program,
            sdf,
            clear_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            draw_rect,
            bound_program: Cell::new(program),
        }
    }

    pub fn gl(&self) -> &glow::Context {
        &self.gl
    }

    pub fn clear_color<C: Into<Color>>(&mut self, color: C) {
        let c = color.into();
        self.clear_color = c;
        unsafe { self.gl.clear_color(c.r, c.g, c.b, c.a) }
    }

    pub fn clear(&self) {
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) }
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        unsafe { self.gl.viewport(0, 0, width as i32, height as i32) }
        self.transform
            .resized(glutin::dpi::PhysicalSize { width, height })
    }

    unsafe fn create_program(
        gl: &glow::Context,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Program {
        let program = gl.create_program().expect("Failed to create program");

        let shader_soruces = [
            (glow::VERTEX_SHADER, vertex_source),
            (glow::FRAGMENT_SHADER, fragment_source),
        ];

        let mut shaders = vec![];
        for (stype, source) in shader_soruces.iter() {
            let shader = gl.create_shader(*stype).expect("Failed to create shader");
            gl.shader_source(shader, source);
            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }

            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        // Shaders are compiled and linked with the program, we don't need them anymore
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        program
    }

    pub fn set_color_uniform(&self, color: Color) {
        self.bind_program();
        unsafe {
            let uniform = self.gl.get_uniform_location(self.program, "Color");
            self.gl
                .uniform_4_f32(uniform.as_ref(), color.r, color.g, color.b, color.a);
        }
    }

    pub fn bind_program(&self) {
        if self.bound_program.get() != self.program {
            unsafe {
                self.gl.use_program(Some(self.program));
            }
            self.bound_program.set(self.program);
        }
    }

    pub fn bind_sdf(&self) {
        if self.bound_program.get() != self.sdf {
            unsafe {
                self.gl.use_program(Some(self.sdf));
            }
            self.bound_program.set(self.sdf);
        }
    }

    //TODO: gen- Make this an enum
    pub fn set_texture_coloring_uniform(&self, value: TextureColoring) {
        self.bind_program();
        let uniform = unsafe { self.gl.get_uniform_location(self.program, "ColorTexture") };

        unsafe {
            let ival = match value {
                TextureColoring::MixTexture => 1,
                TextureColoring::Texture => 0,
                TextureColoring::Color => 2,
            };

            self.gl.uniform_1_i32(uniform.as_ref(), ival);
        }
    }

    pub fn draw_rectangle(&self, pos: Vec2, dim: Vec2) {
        // The rectangle we use to draw, self.draw_rect, spans from (OpenGL Normalized Coordinates)
        // -1,1 to 1,-1. That means any scale we appply via our little uniform will be 2x, as it
        // multiplies both verticies away from the center.

        let gl_pos = self.transform.vec_to_opengl(pos);
        let gl_dim = self.transform.vec_to_opengl(dim / 2);

        unsafe {
            self.bind_program();

            let uniform_position = self.gl.get_uniform_location(self.program, "WorldPosition");
            let uniform_scale = self.gl.get_uniform_location(self.program, "Scale");
            self.gl
                .uniform_2_f32(uniform_position.as_ref(), gl_pos.x, gl_pos.y);
            self.gl
                .uniform_2_f32(uniform_scale.as_ref(), gl_dim.x, gl_dim.y);

            self.draw_rect.bind(&self.gl);
            self.gl
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
        }
    }

    pub fn draw_sdf(&self, sdf: SignedDistance) {
        self.bind_sdf();

        match sdf {
            SignedDistance::Circle {
                center,
                radius,
                color,
            } => unsafe {
                let uniform_color = self.gl.get_uniform_location(self.sdf, "Color");
                let uniform_pointpair = self.gl.get_uniform_location(self.sdf, "PointPair");
                let uniform_drawmethod = self.gl.get_uniform_location(self.sdf, "DrawMethod");

                let pixel_center = self.transform.vec_to_pixels(center);

                self.gl
                    .uniform_4_f32(uniform_color.as_ref(), color.r, color.g, color.b, color.a);
                self.gl.uniform_4_f32(
                    uniform_pointpair.as_ref(),
                    pixel_center.x,
                    pixel_center.y,
                    radius as f32,
                    0.0,
                );
                self.gl
                    .uniform_1_i32(uniform_drawmethod.as_ref(), sdf.draw_method_index());
            },
            SignedDistance::LineSegment {
                start,
                end,
                thickness,
                color,
            } => unsafe {
                let uniform_color = self.gl.get_uniform_location(self.sdf, "Color");
                let uniform_pointpair = self.gl.get_uniform_location(self.sdf, "PointPair");
                let uniform_parameters = self.gl.get_uniform_location(self.sdf, "Parameters");
                let uniform_drawmethod = self.gl.get_uniform_location(self.sdf, "DrawMethod");

                let pixel_start = self.transform.vec_to_pixels(start);
                let pixel_end = self.transform.vec_to_pixels(end);

                self.gl
                    .uniform_4_f32(uniform_color.as_ref(), color.r, color.g, color.b, color.a);
                self.gl.uniform_4_f32(
                    uniform_pointpair.as_ref(),
                    pixel_start.x,
                    pixel_start.y,
                    pixel_end.x,
                    pixel_end.y,
                );
                self.gl
                    .uniform_4_f32(uniform_parameters.as_ref(), thickness as f32, 0.0, 0.0, 0.0);
                self.gl
                    .uniform_1_i32(uniform_drawmethod.as_ref(), sdf.draw_method_index());
            },
        }

        let (pos, dim) = sdf.get_bounds(&self.transform);
        let gl_pos = self.transform.vec_to_opengl(pos);
        let gl_dim = self.transform.vec_to_opengl(dim / 2);

        unsafe {
            let uniform_position = self.gl.get_uniform_location(self.sdf, "WorldPosition");
            let uniform_scale = self.gl.get_uniform_location(self.sdf, "Scale");
            self.gl
                .uniform_2_f32(uniform_position.as_ref(), gl_pos.x, gl_pos.y);
            self.gl
                .uniform_2_f32(uniform_scale.as_ref(), gl_dim.x, gl_dim.y);

            self.draw_rect.bind(&self.gl);
            self.gl
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
        }
    }

    pub fn draw_fullscreen(&self) {
        unsafe {
            self.bind_program();

            let uniform_position = self.gl.get_uniform_location(self.program, "WorldPosition");
            let uniform_scale = self.gl.get_uniform_location(self.program, "Scale");
            self.gl.uniform_2_f32(uniform_position.as_ref(), 0.0, 0.0);
            self.gl.uniform_2_f32(uniform_scale.as_ref(), 1.0, 1.0);

            self.draw_rect.bind(&self.gl);
            self.gl
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
        }
    }
}

impl Drop for OpenGl {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_program(self.sdf);
            self.draw_rect.delete(&self.gl);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TextureColoring {
    MixTexture,
    Texture,
    Color,
}

#[derive(Copy, Clone, Debug)]
pub enum SignedDistance {
    Circle {
        center: Vec2,
        radius: u32,
        color: Color,
    },
    LineSegment {
        start: Vec2,
        end: Vec2,
        thickness: u32,
        color: Color,
    },
}

impl SignedDistance {
    fn draw_method_index(&self) -> i32 {
        match self {
            SignedDistance::Circle { .. } => 1,
            SignedDistance::LineSegment { .. } => 2,
        }
    }

    fn get_bounds(&self, trns: &Transform) -> (Vec2, Vec2) {
        match self {
            SignedDistance::Circle { center, radius, .. } => {
                (*center, Vec2::new(*radius as f32, *radius as f32) * 2)
            }
            SignedDistance::LineSegment {
                start,
                end,
                thickness,
                ..
            } => {
                let mut rstart = Vec2::ZERO;
                let mut rend = Vec2::ZERO;

                if start.x < end.x {
                    rstart.x = start.x;
                    rend.x = end.x;
                } else {
                    rstart.x = end.x;
                    rend.x = start.x;
                }

                if start.y < end.y {
                    rstart.y = start.y;
                    rend.y = end.y;
                } else {
                    rstart.y = end.y;
                    rend.y = start.y;
                }

                let dim = rend - rstart;
                let hdim = dim.abs() / 2;

                (
                    rstart + hdim,
                    hdim * 2
                        + Vec2::new(
                            trns.pixels_to_mur(*thickness),
                            trns.pixels_to_mur(*thickness),
                        ) * 3,
                )
            }
        }
    }

    pub fn line_segment<S, E, C>(start: S, end: E, thickness: u32, color: C) -> SignedDistance
    where
        S: Into<Vec2>,
        E: Into<Vec2>,
        C: Into<Color>,
    {
        SignedDistance::LineSegment {
            start: start.into(),
            end: end.into(),
            thickness,
            color: color.into(),
        }
    }

    pub fn circle<P, C>(center: P, radius: u32, color: C) -> SignedDistance
    where
        P: Into<Vec2>,
        C: Into<Color>,
    {
        SignedDistance::Circle {
            center: center.into(),
            radius,
            color: color.into(),
        }
    }
}
