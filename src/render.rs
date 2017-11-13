use std;
use std::mem::size_of;
use std::ptr;
use gl;
use gl::types::*;

const NUM_RECTANGLES: usize = 8 + 8 + 1;
const NUM_VERTICES: usize = NUM_RECTANGLES*4;
const NUM_INDICES: usize = NUM_RECTANGLES*6;

const COLORS: [[f32; 3]; NUM_RECTANGLES] = [
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0],
    [0.5, 1.0, 0.0],
    [0.5, 0.0, 1.0],
    [0.0, 0.0, 0.5],
    [0.0, 0.5, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
];

pub struct Renderer {
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    vertices: Vec<Vertex>,

    vertex_array: GLuint,

    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,

    scale_uniform: GLint,
}

#[derive(Clone, Default)]
struct Vertex {
    position: [f32; 2],
    step: f32,
    color: [f32; 3],
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

impl Renderer {
    pub unsafe fn new() -> Self {
        let mut vertex_array = 0;
        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);

        let mut vertex_buffer = 0;
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

        let mut index_buffer = 0;
        gl::GenBuffers(1, &mut index_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

        let vertex_shader = compile_shader(include_str!("../shader/shader.vert"), gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(include_str!("../shader/shader.frag"), gl::FRAGMENT_SHADER);
        let program = link_program(vertex_shader, fragment_shader);

        gl::UseProgram(program);

        // Specify the layout of the vertex data
        let attributes = [
            ("in_position\0",    2, offset_of!(Vertex, position)),
            ("in_color\0",       3, offset_of!(Vertex, color)),
            ("in_step\0",        1, offset_of!(Vertex, step)),
        ];

        for &(name, size, offset) in &attributes {
            let attribute = gl::GetAttribLocation(program, name.as_ptr() as *const GLchar);
            gl::EnableVertexAttribArray(attribute as GLuint);
            gl::VertexAttribPointer(attribute as GLuint, size, gl::FLOAT, gl::FALSE as GLboolean, size_of::<Vertex>() as GLsizei, offset as *const _);
        }

        let scale_uniform = gl::GetUniformLocation(program, "scale\0".as_ptr() as *const GLchar);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::FRAMEBUFFER_SRGB); // Linear blending.

        let mut vertices: Vec<Vertex> = std::iter::repeat(Vertex::default()).take(NUM_VERTICES).collect();
        for i in 0..vertices.len() {
            vertices[i].color = COLORS[i/4];
            vertices[i].step = 10000.0;
        }

        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len()*size_of::<Vertex>()) as GLsizeiptr, vertices.as_ptr() as *const _, gl::STREAM_DRAW);

        let indices: Vec<u32> = (0..NUM_RECTANGLES).map(|i| (i*4) as u32).flat_map(|i| vec![i, i+1, i+2, i, i+2, i+3]).collect();
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len()*size_of::<u32>()) as GLsizeiptr, indices.as_ptr() as *const _, gl::STATIC_DRAW);

        Renderer {
            vertex_buffer,
            index_buffer,
            vertices,
            vertex_array,
            vertex_shader,
            fragment_shader,
            program,
            scale_uniform,
        }
    }

    pub unsafe fn draw(&mut self, size: (u32, u32), hidpi_factor: f32) {
        let w = size.0 as f32/hidpi_factor;
        let h = size.1 as f32/hidpi_factor;

        let mut i = 0;

        let border_width = 1.0;
        self.vertices[i + 0].position = [0.0, 0.0];
        self.vertices[i + 1].position = [border_width, 0.0];
        self.vertices[i + 2].position = [border_width, h];
        self.vertices[i + 3].position = [0.0, h];
        i += 4;

        self.vertices[i + 0].position = [w, 0.0];
        self.vertices[i + 1].position = [w - border_width, 0.0];
        self.vertices[i + 2].position = [w - border_width, h];
        self.vertices[i + 3].position = [w, h];
        i += 4;

        self.vertices[i + 0].position = [0.0, 0.0];
        self.vertices[i + 1].position = [0.0, border_width];
        self.vertices[i + 2].position = [w, border_width];
        self.vertices[i + 3].position = [w, 0.0];
        i += 4;

        self.vertices[i + 0].position = [0.0, h];
        self.vertices[i + 1].position = [0.0, h - border_width];
        self.vertices[i + 2].position = [w, h - border_width];
        self.vertices[i + 3].position = [w, h];
        i += 4;

        // corners
        let corner_size = 50.0;
        for &position in &[[0.0, 0.0], [w - corner_size,  0.0], [w - corner_size, h - corner_size], [0.0, h - corner_size]] {
            i = self.add_square(i, position, corner_size);
        }

        let block_size = 64.0;
        let block_scale = 300.0;
        let blocks_origin = [corner_size + 10.0, corner_size + 10.0];

        let position = [blocks_origin[0] + block_scale*(hidpi_factor - 1.0), blocks_origin[1] + block_size*1.2];
        for j in 0..4 {
            self.vertices[i + j].step = 1.0/hidpi_factor;
        }
        i = self.add_square(i, position, block_size);

        for &hidpi_factor in &[1.0, 1.25, 1.5, 2.0, 2.25, 2.5, 3.0, 4.0] {
            let position = [blocks_origin[0] + block_scale*(hidpi_factor - 1.0), blocks_origin[1]];
            for j in 0..4 {
                self.vertices[i + j].step = 1.0/hidpi_factor;
                if hidpi_factor.fract() == 0.0 {
                    self.vertices[i + j].color = [0.0, 0.0, 0.3];
                } else {
                    self.vertices[i + j].color = [0.0, 0.0, 0.0];
                }
            }
            i = self.add_square(i, position, block_size);
        }

        gl::Uniform2f(self.scale_uniform, 2.0/w, -2.0/h);

        gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vertices.len()*size_of::<Vertex>()) as GLsizeiptr, self.vertices.as_ptr() as *const _);
        gl::DrawElements(gl::TRIANGLES, NUM_INDICES as GLsizei, gl::UNSIGNED_INT, ptr::null());
    }

    fn add_square(&mut self, i: usize, position: [f32; 2], size: f32) -> usize {
        self.vertices[i + 0].position = position;
        self.vertices[i + 1].position = [position[0] + size, position[1]];
        self.vertices[i + 2].position = [position[0] + size, position[1] + size];
        self.vertices[i + 3].position = [position[0], position[1] + size];
        i + 4
    }

    pub unsafe fn cleanup(self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fragment_shader);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteBuffers(2, [self.vertex_buffer, self.index_buffer].as_ptr());
        gl::DeleteVertexArrays(1, &self.vertex_array);
    }
}

unsafe fn compile_shader(source: &str, kind: GLenum) -> GLuint {
    let shader = gl::CreateShader(kind);

    // Attempt to compile the shader.
    gl::ShaderSource(shader, 1, [source.as_ptr() as *const GLchar].as_ptr(), [source.len() as GLint].as_ptr());
    gl::CompileShader(shader);

    // Get the compile status.
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error.
    if status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len((log_length as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(shader, log_length, ptr::null_mut(), log.as_mut_ptr() as *mut GLchar);

        panic!("OpenGL shader compilation failed: {}", std::str::from_utf8(&log).unwrap());
    }

    shader
}

unsafe fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl::CreateProgram();

    gl::AttachShader(program, vertex_shader);
    gl::AttachShader(program, fragment_shader);
    gl::LinkProgram(program);

    // Get the link status.
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error.
    if status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len((log_length as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program, log_length, ptr::null_mut(), log.as_mut_ptr() as *mut GLchar);

        panic!("OpenGL program linking failed: {}", std::str::from_utf8(&log).unwrap());
    }

    program
}
