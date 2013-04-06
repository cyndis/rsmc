use glfw;
use glcore::*;

use shader::Program;
use buffer::Buffer;
use texture::Texture;
use chunk;
use chunk::Chunk;

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

fn perspective(fov_y: float, aspect: float, z_near: float, z_far: float) -> Mat4f {
    let f = 1.0 / float::tan(0.5 * fov_y);
    BaseMat4::new(f/aspect, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0,
                  (z_far + z_near) / (z_near - z_far), -1.0, 0.0, 0.0,
                  (2.0 * z_far * z_near) / (z_near - z_far), 0.0)
}

static MOVE_SPEED: float = 5.0f;

fn main() {
    glfw::set_error_callback(error_cb);

    do glfw::spawn {
        let wnd = glfw::Window::create(1280, 800, "Kato moro", glfw::Windowed).unwrap();

        wnd.make_context_current();
        wnd.set_key_callback(key_cb);
        wnd.set_input_mode(glfw::CURSOR_MODE, glfw::CURSOR_CAPTURED as int);

        glfw::set_swap_interval(1);

        glDebugMessageCallback(debug_cb, ptr::null());
        glEnable(GL_DEBUG_OUTPUT);
        glEnable(GL_CULL_FACE);
        glEnable(GL_DEPTH_TEST);
        glDepthFunc(GL_LEQUAL);

        let mut game = GameState {
            world: Chunk::new(),
            position: BaseVec3::new(0.0, 0.0, 0.0),
            rot_x: 0.0, rot_y: 0.0
        };

        let mut state = initialize_opengl(&mut game);
        let mut camera = CameraState {
            position: game.position.add_v(&BaseVec3::new(0.0, 2.5, 0.0)),/*BaseVec3::new(1.05, -0.46, -29.3)*/
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

            let rot_hori = Quat::from_angle_axis(game.rot_x, &BaseVec3::new(0.0, 1.0, 0.0));
            let rot_vert = Quat::from_angle_axis(game.rot_y, &BaseVec3::new(1.0, 0.0, 0.0));

            let plane_fwd = rot_hori.mul_v(&BaseVec3::new(0.0, 0.0, -1.0));
            let fwd       = camera.rotation.mul_v(&BaseVec3::new(0.0, 0.0, -1.0));
            let up        = camera.rotation.mul_v(&BaseVec3::new(0.0, 1.0, 0.0));
            let rt        = fwd.cross(&up);

            if wnd.get_key(glfw::KEY_A) == glfw::PRESS {
                camera.position.add_self_v(&rt.mul_t(-dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_D) == glfw::PRESS {
                camera.position.add_self_v(&rt.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_W) == glfw::PRESS {
                camera.position.add_self_v(&plane_fwd.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_S) == glfw::PRESS {
                camera.position.add_self_v(&plane_fwd.mul_t(-dt*MOVE_SPEED));
            }

            let cursor = wnd.get_cursor_pos();
            let (dx, dy) = match (cursor, last_cursor) { ((a,b),(c,d)) => (a-c,b-d) };
            last_cursor = cursor;

            game.rot_x -= (dx as float / 2800.0) * (3.1416 / 2.0);
            game.rot_y -= (dy as float / 3800.0) * (3.1416 / 2.0);

            camera.rotation = rot_hori.mul_q(&rot_vert);

            draw(&mut state, &camera, &game);

            wnd.swap_buffers();

//            io::println(fmt!("%?", camera.position));
        }
    }
}

struct RendererState {
    program: Program,
    brick_tex: Texture,
}

struct CameraState {
    position: Vec3f,
    rotation: Quatf
}

struct GameState {
    world: Chunk,
    position: Vec3f,
    rot_x: float, rot_y: float
}

static vertex_shader2: &'static str = "
#version 330
in vec3 position;
in vec2 texcoord;
in vec3 normal;
uniform mat4 projection;
uniform mat4 modelview;

out vec2 v_texcoord;
out vec3 v_position;

out vec4 lieye;
out vec4 vneye;

void main() {
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_texcoord = texcoord;
    v_position = position;

    lieye = modelview * vec4(2.0, 1.0, 1.0, 0.0);
    vneye = modelview * vec4(normal, 0.0);
}
";

static fragment_shader2: &'static str = "
#version 330
layout (location = 0) out vec4 outputColor;
uniform sampler2D texture;

in vec2 v_texcoord;
in vec3 v_position;

in vec4 lieye;
in vec4 vneye;

void main() {
    vec4 Ld = texture2D(texture, v_texcoord);

    vec4 n_eye = normalize(vneye);

    vec4 Ia = vec4(0.13, 0.13, 0.13, 1.0);
    vec4 Id = vec4(0.75, 0.75, 0.75, 1.0) * max(dot(lieye, n_eye), 0.0);
    outputColor = Ld * (Ia + Id);
}
";

fn initialize_opengl(game: &mut GameState) -> RendererState {
    glViewport(0, 0, 1280, 800);

    let chunk = &mut game.world;
    for chunk.each_block_mut |(x,y,z), block| {
        if y == 15 { *block = chunk::Air };
        if x < 15 && x > 0 && z < 15 && z > 0 { *block = chunk::Air };
        if y == 0 { *block = chunk::Brick };
    }
    chunk.update_buffer_cache();

    let mut program = Program::new(vertex_shader2, fragment_shader2);

    // lmath's perspective function gives wrong values
    //let projection = lmath::projection::perspective(65.0 / 180.0 * 3.1416, 800.0 / 480.0, 0.1, 60.0);
    //let projection = BaseMat4::new(0.9418, 0.0, 0.0, 0.0, 0.0, 1.5696, 0.0, 0.0, 0.0, 0.0, -1.003, -1.0, 0.0, 0.0, -0.20003, 0.0);
    let projection = perspective(67.5 / 180.0 * 3.1416, 800.0 / 480.0, 0.1, 60.0);
    io::println(fmt!("%?", projection));
    program.set_uniform_mat4("projection", &projection);

    RendererState {
        program: program,
        brick_tex: Texture::load_file(~"brick.png").unwrap(),
    }
}

fn draw(state: &mut RendererState, camera: &CameraState, game: &GameState) {
    let camera_matrix: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                             0.0, 1.0, 0.0, 0.0,
                                             0.0, 0.0, 1.0, 0.0,
                                             -camera.position.x, -camera.position.y,
                                             -camera.position.z, 1.0
                                         );
    let camera_matrix = camera.rotation.inverse().to_mat3().to_mat4().mul_m(&camera_matrix);


    let (x, y, z) = (-8.0, 0.0, -8.0);
    let modelview: Mat4f = BaseMat4::new(1.0, 0.0, 0.0, 0.0,
                                         0.0, 1.0, 0.0, 0.0,
                                         0.0, 0.0, 1.0, 0.0,
                                         x,   y,   z,   1.0);

    let modelview = camera_matrix.mul_m(&modelview);

    state.program.bind();
    state.program.set_uniform_mat4("modelview", &modelview);

    state.brick_tex.bind(0);
    state.program.set_uniform_int("texture", 0);

    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    game.world.draw_cached(&mut state.program);
}
