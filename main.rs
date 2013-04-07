use glfw;
use glcore::*;

use shader::Program;
use buffer::Buffer;
use texture;
use texture::Texture;
use chunk;
use chunk::Chunk;
use font;
use font::Font;

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
        glClearColor(0.53, 0.81, 0.98, 1.0);

        let mut game = GameState {
            world: Chunk::new(),
            position: BaseVec3::new(8.0, 1.0, 8.0),
            rot_x: 0.0, rot_y: 0.0,
            target: NumVec::zero(),
            vel_y: 0.0
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

            /* collision detection now works, but if we get too close, the
             * the wall is clipped by the near clipping plane
             */

            let mut target_pos: Vec3f = NumVec::zero();
            if wnd.get_key(glfw::KEY_A) == glfw::PRESS {
                target_pos.add_self_v(&rt.mul_t(-dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_D) == glfw::PRESS {
                target_pos.add_self_v(&rt.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_W) == glfw::PRESS {
                target_pos.add_self_v(&plane_fwd.mul_t(dt*MOVE_SPEED));
            }
            if wnd.get_key(glfw::KEY_S) == glfw::PRESS {
                target_pos.add_self_v(&plane_fwd.mul_t(-dt*MOVE_SPEED));
            }


            let stop_fall;
            game.vel_y -= 30.0 * dt;
            {
            let below_pos = BaseVec3::new(game.position.x, game.position.y-0.001, game.position.z);
            let block_below = game.world.block_at(&below_pos);
            stop_fall =
                match block_below {
                    Some(&chunk::Air) => false,
                    _ => true
                };
            }
            if stop_fall { game.vel_y = 0.0; }

            if wnd.get_key(glfw::KEY_SPACE) == glfw::PRESS && stop_fall {
                game.vel_y = 8.0;
            }

            let target_xv = BaseVec3::new(target_pos.x, 0.0, 0.0);
            let target_yv = BaseVec3::new(0.0, game.vel_y*dt, 0.0);
            let target_zv = BaseVec3::new(0.0, 0.0, target_pos.z);

            let abs_xv = game.position.add_v(&target_xv);
            let abs_yv = game.position.add_v(&target_yv);
            let abs_zv = game.position.add_v(&target_zv);

            let rem_xm = if target_xv.x < 0.0 { game.position.x as int as float - game.position.x }
                         else { game.position.x as int as float - game.position.x + 0.9999 };
            let rem_ym = if target_yv.y < 0.0 { game.position.y as int as float - game.position.y }
                         else { game.position.y as int as float - game.position.y + 0.9999 };
            let rem_zm = if target_zv.z < 0.0 { game.position.z as int as float - game.position.z }
                         else { game.position.z as int as float - game.position.z + 0.9999 };

            /* there is probably a bug here allowing entering walls.. */

            game.position.add_self_v(&
                match game.world.block_at(&abs_xv) {
                    Some(&chunk::Air) => target_xv,
                    _ => BaseVec3::new(rem_xm, 0.0, 0.0)
                }
            );

            game.position.add_self_v(&
                match game.world.block_at(&abs_yv) {
                    Some(&chunk::Air) => target_yv,
                    _ => BaseVec3::new(0.0, rem_ym, 0.0)
                }
            );

            game.position.add_self_v(&
                match game.world.block_at(&abs_zv) {
                    Some(&chunk::Air) => target_zv,
                    _ => BaseVec3::new(0.0, 0.0, rem_zm)
                }
            );

            let cursor = wnd.get_cursor_pos();
            let (dx, dy) = match (cursor, last_cursor) { ((a,b),(c,d)) => (a-c,b-d) };
            last_cursor = cursor;

            game.rot_x -= (dx as float / 2800.0) * (3.1416 / 2.0);
            game.rot_y -= (dy as float / 3800.0) * (3.1416 / 2.0);

            camera.rotation = rot_hori.mul_q(&rot_vert);
            camera.position = game.position.add_v(&BaseVec3::new(0.0, 2.5, 0.0));

            draw(&mut state, &camera, &game);

            wnd.swap_buffers();

//            io::println(fmt!("%?", camera.position));
        }
    }
}

struct RendererState {
    program: Program,
    brick_tex: Texture,
    font: Font
}

struct CameraState {
    position: Vec3f,
    rotation: Quatf
}

struct GameState {
    world: Chunk,
    position: Vec3f,
    rot_x: float, rot_y: float,
    target: Vec3f,
    vel_y: float
}

fn initialize_opengl(game: &mut GameState) -> RendererState {
    glViewport(0, 0, 1280, 800);

    let chunk = &mut game.world;
    for chunk.each_block_mut |(x,y,z), block| {
        if y == 15 { *block = chunk::Air };
        if x < 15 && x > 0 && z < 15 && z > 0 { *block = chunk::Air };
        if y == 0 { *block = chunk::Brick };

        if (x,y,z) == (4,1,4) { *block = chunk::Brick };
        if (x,y,z) == (4,1,5) { *block = chunk::Brick };
        if (x,y,z) == (4,2,5) { *block = chunk::Brick };
    }
    chunk.update_buffer_cache();

    let mut program = Program::new(io::read_whole_file_str(&path::Path("shader.vert")).unwrap(),
                                   io::read_whole_file_str(&path::Path("shader.frag")).unwrap());

    let projection = lmath::projection::perspective(67.5, 800.0 / 480.0, 0.1, 60.0);
    program.set_uniform_mat4("projection", &projection);

    RendererState {
        program: program,
        brick_tex: Texture::load_file(~"texes.png", texture::TextureArray(2)).unwrap(),
        font: Font::new(~"font.png", ~"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890{}[]()<>$*-+=/#_%^@\\&|~?'\"!,.;:")
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


    let (x, y, z) = (0.0, 0.0, 0.0);
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

    state.font.draw(fmt!("B %? VY %? P %?", game.world.block_at(&game.position), game.vel_y, game.position));
}
