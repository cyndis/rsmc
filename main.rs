use glfw;
use glcore::*;

use shader::Program;
use buffer::Buffer;

use common::*;

use lmath;
// for Mat3::to_mat4
use lmath::mat::BaseMat3;

fn error_cb(_error: libc::c_int, desc: ~str) {
    println(fmt!("GLFW error: %s", desc));
}

fn key_cb(window: &glfw::Window, key: libc::c_int, action: libc::c_int) {
    if action == glfw::PRESS && key == glfw::KEY_ESCAPE {
        window.set_should_close(true);
    }
}

extern "C"
fn debug_cb(_source: u32, _type: u32, _id: u32, _severity: u32, length: u32,
            message: *u8, _param: *libc::c_void)
{
    let buf = unsafe { vec::from_buf(message, length as uint) };
    let string = str::from_bytes(buf);
    println(string);
}

fn main() {
    glfw::set_error_callback(error_cb);

    do glfw::spawn {
        let wnd = glfw::Window::create(800, 480, "Kato moro", glfw::Windowed).unwrap();

        wnd.make_context_current();
        wnd.set_key_callback(key_cb);
        glfw::set_swap_interval(1);

        glDebugMessageCallback(debug_cb, ptr::null());
        glEnable(GL_DEBUG_OUTPUT);

        glDisable(GL_CULL_FACE);

        let mut state = initialize_opengl();

        println("-- INITIALIZED --");

        while !wnd.should_close() {
            glfw::poll_events();

            draw(&mut state);

            wnd.swap_buffers();

            println("-- FRAME BOUNDARY --");
        }
    }
}

struct RendererState {
    program: Program,
    vbo: Buffer
}

static vertex_shader: &'static str = "
#version 330
in vec3 position;
uniform mat4 projection;
uniform mat4 modelview;

void main() {
    gl_Position = projection * modelview * vec4(position, 1.0);
}
";

static fragment_shader: &'static str = "
#version 330
layout (location = 0) out vec4 outputColor;

void main() {
    outputColor = vec4(1.0, 0.5, 0.0, 1.0);
}
";

fn initialize_opengl() -> RendererState {
    glViewport(0, 0, 800, 480);

    let triangle: &[Vec3f] = [
        BaseVec3::new( 0.5, 0.0, 0.0),
        BaseVec3::new(-0.5, 1.0, 0.0),
        BaseVec3::new(-0.5,-1.0, 0.0)
    ];

    let mut buffer = Buffer::new();
    buffer.update(triangle);

    let mut program = Program::new(vertex_shader, fragment_shader);

    let projection = lmath::projection::perspective(3.1416 / 2.0, 800.0 / 480.0, 0.1, 200.0);
    program.set_uniform_mat4("projection", &projection);

    RendererState {
        program: program,
        vbo: buffer
    }
}

fn draw(state: &mut RendererState) {
    state.program.bind();
    state.program.set_attribute_vec3("position", &state.vbo);

    let (x, y, z) = (0.0, 0.0, -50.0);
    let modelview: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                         0.0, 1.0, 0.0, 0.0,
                                         0.0, 0.0, 1.0, 0.0,
                                         x,   y,   z,   1.0);

    let angle = glfw::get_time() as float % (2.0*3.1416);
    let rotation = Quat::from_angle_y(angle).to_mat3().to_mat4();
    let modelview = modelview.mul_m(&rotation);

    state.program.set_uniform_mat4("modelview", &modelview);

    glClear(GL_COLOR_BUFFER_BIT);

    glDrawArrays(GL_TRIANGLES, 0, 3);
}
