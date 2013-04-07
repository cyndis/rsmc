use texture;
use texture::Texture;

use shader::Program;

use buffer;
use buffer::Buffer;

use glcore::*;
use common::*;

use lmath;

struct Font {
   texture: Texture,
   program: Program,
   map: ~str
}

pub impl Font {
    fn new(path: ~str, map: ~str) -> Font {
        Font {
            texture: Texture::load_file(path, texture::TextureArray(31*3)).unwrap(),
            program: Program::new(vertex_shader, fragment_shader),
            map: map
        }
    }

    fn draw(&self, message: &str) {
        let mut vbuf = Buffer::new();
        let mut tbuf = Buffer::new();

        let mut vs = ~[];
        let mut ts = ~[];

        let mut drawn_chars = 0;
        for message.each_chari |index, ch| {
            match str::find_char(self.map, ch) {
                Some(map_index) => {
                    vs.push_all_move(make_cube(index as float * 0.6, 0.0));
                    ts.push_all_move(make_cube_texcoord(map_index));
                    drawn_chars += 1;
                },
                None => ()
            }
        }

        /*
        io::println(fmt!("vs %?", vs));
        io::println(fmt!("ts %?", ts));
        */

        vbuf.update(vs);
        tbuf.update(ts);

        self.program.bind();

        self.program.set_attribute_vec3("position", &vbuf);
        self.program.set_attribute_vec3("texcoord", &tbuf);

        let projection = lmath::projection::orthographic(0.0, 33.3, 0.0, 20.0, -1.0, 1.0);
        self.program.set_uniform_mat4("projection", &projection);
        self.program.set_uniform_mat4("modelview", &BaseMat::identity());

        self.texture.bind(0);
        self.program.set_uniform_int("texture", 0);

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        glDrawArrays(GL_QUADS, 0, (drawn_chars * 4) as i32);

        // unbind so that we aren't bound to a deleted buffer
        glBindBuffer(GL_ARRAY_BUFFER, 0);
    }
}

static vertex_shader: &'static str = "
#version 330
in vec3 position;
in vec3 texcoord;
uniform mat4 projection;
uniform mat4 modelview;

out vec3 v_texcoord;

void main() {
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_texcoord = texcoord;
}
";

static fragment_shader: &'static str = "
#version 330
#extension GL_EXT_texture_array : enable
layout (location = 0) out vec4 outputColor;
uniform sampler2DArray texture;

in vec3 v_texcoord;

void main() {
    vec4 color = texture2DArray(texture, v_texcoord);
    if (color == vec4(1.0, 1.0, 1.0, 1.0))
        color = vec4(0.0, 0.0, 0.0, 0.0);
    else
        color = vec4(1.0, 1.0, 1.0, 1.0);
    outputColor = color;
}
";

fn make_cube(x: float, y: float) -> ~[Vec3f] {
    ~[
        BaseVec3::new(x, y, 0.0), BaseVec3::new(x+0.5, y, 0.0),
        BaseVec3::new(x+0.5, y+1.0, 0.0), BaseVec3::new(x, y+1.0, 0.0)
    ]
}

// real font size is: 5*10
fn make_cube_texcoord(ch: uint) -> ~[Vec3f] {
    let ch = ch as float;
    ~[
        BaseVec3::new(0.0, 0.625, ch), BaseVec3::new(0.625, 0.625, ch),
        BaseVec3::new(0.625, 0.0, ch), BaseVec3::new(0.0, 0.0, ch)
    ]
}
