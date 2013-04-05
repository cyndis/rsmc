use glfw;
use glcore::*;

use shader::Program;
use buffer::Buffer;
use texture::Texture;

use common::*;

use lmath;
// for Mat3::to_mat4
use lmath::mat::BaseMat3;
use lmath::vec::{AffineVec, NumVec, NumVec3};

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

fn add_quat(a: &Quatf, b: &Quatf) -> Quatf {
    Quat::from_sv(a.s*b.s - a.v.dot(&b.v),
                  b.v.mul_t(a.s).add_v(&a.v.mul_t(b.s)).add_v(&a.v.cross(&b.v)))
}

fn main() {
    glfw::set_error_callback(error_cb);

    do glfw::spawn {
        let wnd = glfw::Window::create(800, 480, "Kato moro", glfw::Windowed).unwrap();

        wnd.make_context_current();
        wnd.set_key_callback(key_cb);
        wnd.set_input_mode(glfw::CURSOR_MODE, glfw::CURSOR_CAPTURED as int);

        glfw::set_swap_interval(1);

        glDebugMessageCallback(debug_cb, ptr::null());
        glEnable(GL_DEBUG_OUTPUT);

        glEnable(GL_CULL_FACE);

        let mut state = initialize_opengl();

        println("-- INITIALIZED --");

        let mut last_cursor = wnd.get_cursor_pos();

        while !wnd.should_close() {
            glfw::poll_events();

            let cursor = wnd.get_cursor_pos();
            let (dx, dy) = match (cursor, last_cursor) { ((a,b),(c,d)) => (a-c,b-d) };
            last_cursor = cursor;

            state.rota += dx as float / 150.0;
            state.rotb += dy as float / 150.0;

            state.rotation = add_quat(
                &Quat::from_angle_axis(state.rota, &BaseVec3::new(0.0, 1.0, 0.0)),
                &Quat::from_angle_axis(state.rotb, &BaseVec3::new(1.0, 0.0, 0.0))
                             );

            /*
            state.rotation = add_quat(
                &state.rotation,
                &Quat::from_angle_axis(0.01*2.0*3.1416, &BaseVec3::new(0.0, 1.0, 0.0)).mul_t(dx as float)
            );
            */

            draw(&mut state);

            wnd.swap_buffers();
        }
    }
}

struct RendererState {
    program: Program,
    vbo: Buffer,
    tbo: Buffer,
    brick_tex: Texture,
    rotation: Quatf,
    rota: float, rotb: float
}

static vertex_shader: &'static str = "
#version 330
in vec3 position;
in vec2 texcoord;
uniform mat4 projection;
uniform mat4 modelview;

out vec2 v_texcoord;

void main() {
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_texcoord = texcoord;
}
";

static fragment_shader: &'static str = "
#version 330
layout (location = 0) out vec4 outputColor;
uniform sampler2D texture;

in vec2 v_texcoord;

void main() {
    outputColor = texture2D(texture, v_texcoord);
}
";

fn initialize_opengl() -> RendererState {
    glViewport(0, 0, 800, 480);

    /*
    let triangle: &[Vec3f] = [
        BaseVec3::new( 0.5, 0.0, 0.0),
        BaseVec3::new(-0.5, 1.0, 0.0),
        BaseVec3::new(-0.5,-1.0, 0.0)
    ];
    */
    let cube = make_cube(0.0, 0.0, 0.0, 0.1);

    let mut buffer = Buffer::new();
    buffer.update(cube);

    let mut tc_buffer = Buffer::new();
    tc_buffer.update(make_cube_texcoord());

    let mut program = Program::new(vertex_shader, fragment_shader);

    let projection = lmath::projection::perspective(3.1416 / 2.0, 800.0 / 480.0, 0.1, 200.0);
    program.set_uniform_mat4("projection", &projection);

    RendererState {
        program: program,
        vbo: buffer,
        tbo: tc_buffer,
        brick_tex: Texture::load_file(~"brick.png").unwrap(),
        rotation: Quat::identity(),
        rota: 0.0, rotb: 0.0
    }
}

fn draw(state: &mut RendererState) {
    state.program.bind();
    state.program.set_attribute_vec3("position", &state.vbo);
    state.program.set_attribute_vec2("texcoord", &state.tbo);

    let (x, y, z) = (0.0, 0.0, -50.0);
    let modelview: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                         0.0, 1.0, 0.0, 0.0,
                                         0.0, 0.0, 1.0, 0.0,
                                         x,   y,   z,   1.0);

    let rotation = state.rotation.to_mat3().to_mat4();
    let modelview = modelview.mul_m(&rotation);

    state.program.set_uniform_mat4("modelview", &modelview);

    state.brick_tex.bind(0);
    state.program.set_uniform_int("texture", 0);

    glClear(GL_COLOR_BUFFER_BIT);

    glDrawArrays(GL_QUADS, 0, 24);
}

fn make_cube(x: float, y: float, z: float, n: float) -> ~[Vec3f] {
    ~[
        BaseVec3::new(x-n,y+n,z-n), BaseVec3::new(x-n,y+n,z+n), BaseVec3::new(x+n,y+n,z+n), BaseVec3::new(x+n,y+n,z-n),  // top
        BaseVec3::new(x-n,y-n,z-n), BaseVec3::new(x+n,y-n,z-n), BaseVec3::new(x+n,y-n,z+n), BaseVec3::new(x-n,y-n,z+n),  // bottom
        BaseVec3::new(x-n,y-n,z-n), BaseVec3::new(x-n,y-n,z+n), BaseVec3::new(x-n,y+n,z+n), BaseVec3::new(x-n,y+n,z-n),  // left
        BaseVec3::new(x+n,y-n,z+n), BaseVec3::new(x+n,y-n,z-n), BaseVec3::new(x+n,y+n,z-n), BaseVec3::new(x+n,y+n,z+n),  // right
        BaseVec3::new(x-n,y-n,z+n), BaseVec3::new(x+n,y-n,z+n), BaseVec3::new(x+n,y+n,z+n), BaseVec3::new(x-n,y+n,z+n),  // front
        BaseVec3::new(x+n,y-n,z-n), BaseVec3::new(x-n,y-n,z-n), BaseVec3::new(x-n,y+n,z-n), BaseVec3::new(x+n,y+n,z-n),  // back
    ]
}

fn make_cube_texcoord() -> ~[Vec2f] {
    ~[
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
        BaseVec2::new(0.0, 0.0), BaseVec2::new(0.0, 1.0), BaseVec2::new(1.0, 1.0), BaseVec2::new(1.0, 0.0),
    ]
}
