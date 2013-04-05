use glfw;
use glcore::*;

use shader::Program;
use buffer::Buffer;
use texture::Texture;

use common::*;

use lmath;
// for Mat3::to_mat4
use lmath::mat::BaseMat3;
use lmath::vec::{AffineVec, NumVec, NumVec3, BaseVec};

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

static MOVE_SPEED: float = 2.5f;

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
        let mut camera = CameraState {
            position: NumVec::zero(),
            rot_x: 0.0,
            rot_y: 0.0,
            rotation: Quat::identity()
        };

        println("-- INITIALIZED --");

        let mut last_cursor = wnd.get_cursor_pos();
        let mut last_update = glfw::get_time();

        while !wnd.should_close() {
            glfw::poll_events();

            let time = glfw::get_time();
            let dt = (time - last_update) as float;
            last_update = time;

            let fwd = camera.rotation.mul_v(&BaseVec3::new(0.0, 0.0, -1.0));
            let up  = camera.rotation.mul_v(&BaseVec3::new(0.0, 1.0, 0.0));
            let rt  = fwd.cross(&up);

            if wnd.get_key(glfw::KEY_A) == glfw::PRESS {
                camera.position.add_self_v(&rt.mul_t(-dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_D) == glfw::PRESS {
                camera.position.add_self_v(&rt.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_W) == glfw::PRESS {
                camera.position.add_self_v(&fwd.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_S) == glfw::PRESS {
                camera.position.add_self_v(&fwd.mul_t(-dt*MOVE_SPEED));
            }

            let cursor = wnd.get_cursor_pos();
            let (dx, dy) = match (cursor, last_cursor) { ((a,b),(c,d)) => (a-c,b-d) };
            last_cursor = cursor;

            camera.rot_x -= (dx as float / 5800.0) * (3.1416 / 2.0);
            camera.rot_y -= (dy as float / 5800.0) * (3.1416 / 2.0);

            camera.rotation = add_quat(
                &Quat::from_angle_axis(camera.rot_x, &BaseVec3::new(0.0, 1.0, 0.0)),
                &Quat::from_angle_axis(camera.rot_y, &BaseVec3::new(1.0, 0.0, 0.0))
                             );

            draw(&mut state, &camera);

            wnd.swap_buffers();
        }
    }
}

struct RendererState {
    program: Program,
    vbo: Buffer,
    tbo: Buffer,
    brick_tex: Texture,
}

struct CameraState {
    position: Vec3f,
    rot_x: float, rot_y: float,
    rotation: Quatf
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
    }
}

fn draw(state: &mut RendererState, camera: &CameraState) {
    state.program.bind();
    state.program.set_attribute_vec3("position", &state.vbo);
    state.program.set_attribute_vec2("texcoord", &state.tbo);

    let camera_matrix: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                             0.0, 1.0, 0.0, 0.0,
                                             0.0, 0.0, 1.0, 0.0,
                                             -camera.position.x, -camera.position.y,
                                             -camera.position.z, 1.0
                                         );
    let camera_matrix = camera.rotation.inverse().to_mat3().to_mat4().mul_m(&camera_matrix);


    let (x, y, z) = (0.0, 0.0, -10.0);
    let modelview: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                         0.0, 1.0, 0.0, 0.0,
                                         0.0, 0.0, 1.0, 0.0,
                                         x,   y,   z,   1.0);

    let modelview = camera_matrix.mul_m(&modelview);

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
