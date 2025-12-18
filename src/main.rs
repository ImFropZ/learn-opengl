extern crate gl;
extern crate glfw;

use std::fs::read_to_string;

use glfw::{Action, Context, Key};

enum ShaderType {
    None,
    Vertex,
    Fragment,
}

struct ShaderSource {
    vertex_source: String,
    fragment_source: String,
}

impl ShaderSource {
    fn new() -> ShaderSource {
        ShaderSource {
            vertex_source: String::new(),
            fragment_source: String::new(),
        }
    }
}

fn read_shader_source(filepath: &str) -> ShaderSource {
    let mut shader_type = ShaderType::None;
    let mut sources: ShaderSource = ShaderSource::new();
    for line in read_to_string(filepath).unwrap().lines() {
        if line.contains("#shader") {
            if line.contains("#shader vertex") {
                shader_type = ShaderType::Vertex;
            } else if line.contains("#shader fragment") {
                shader_type = ShaderType::Fragment;
            } else {
                shader_type = ShaderType::None;
            }
            continue;
        }

        match shader_type {
            ShaderType::Vertex => sources.vertex_source.push_str(&format!("{}\n", line)),
            ShaderType::Fragment => sources.fragment_source.push_str(&format!("{}\n", line)),
            ShaderType::None => {}
        }
    }

    return sources;
}

fn compile_shader(shader_type: u32, source: &str) -> u32 {
    unsafe {
        let id = gl::CreateShader(shader_type);
        let c_str = std::ffi::CString::new(source.as_bytes()).unwrap();

        gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(id);

        let mut result: i32 = 0;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
        if result == gl::FALSE.into() {
            let mut length: i32 = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);
            let mut buffer: Vec<u8> = vec![0; length as usize];
            gl::GetShaderInfoLog(
                id,
                length,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut i8,
            );
            let error = String::from_utf8_lossy(&buffer);
            println!("Failed to compile shader: {}", error);
            gl::DeleteShader(id);
            return 0;
        }

        return id;
    }
}

fn create_shader(vertex_shader: &str, fragment_shader: &str) -> u32 {
    unsafe {
        let program = gl::CreateProgram();
        let vs = compile_shader(gl::VERTEX_SHADER, vertex_shader);
        let fs = compile_shader(gl::FRAGMENT_SHADER, fragment_shader);

        if vs == 0 || fs == 0 {
            println!("Shader compilation failed!");
            return 0;
        }

        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut status: i32 = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == gl::FALSE.into() {
            println!("Program link failed!");
        }

        gl::ValidateProgram(program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        return program;
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));

    let (mut window, events) = glfw
        .create_window(600, 600, "OpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|symbol| {
        window
            .get_proc_address(symbol)
            .map_or(std::ptr::null(), |f| f as *const _)
    });

    let positions: Vec<f32> = vec![
        // First triangle for square
        -0.5, -0.5,
        0.5, -0.5,
        0.5, 0.5,

        // Second triangle for square
        0.5, 0.5,
        -0.5, 0.5,
        -0.5, -0.5
    ];

    let shader: u32;
    unsafe {
        // Create VAO
        let mut vao: u32 = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut buffer: u32 = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (positions.len() * std::mem::size_of::<f32>()) as isize,
            positions.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        let source = read_shader_source("resources/shaders/basic.shader");
        shader = create_shader(&source.vertex_source, &source.fragment_source);
        gl::UseProgram(shader);

        println!("Shader program: {}", shader);
        gl::Viewport(0, 0, 600, 600);
    }

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {}
            }
        }
    }

    unsafe {
        gl::DeleteProgram(shader);
    }
}
