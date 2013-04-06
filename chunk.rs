use common::*;
use glcore::*;
use buffer::Buffer;
use shader::Program;

pub enum Block {
    Air,
    Brick
}

struct BufferCache {
    position: Buffer,
    texcoord: Buffer,
    normal: Buffer,
    vertex_no: uint
}

// 16x16x16 chunk
pub struct Chunk {
    blocks: [Block, ..16*16*16],
    buffer_cache: Option<BufferCache>
}

pub impl Chunk {
    fn new() -> Chunk {
        Chunk {
            blocks: [Brick, ..16*16*16],
            buffer_cache: None
        }
    }

    // x,z is the horizontal plane
    fn each_block(&self, f: &fn(pos: (uint, uint, uint), block: &Block) -> bool) {
        for uint::range(0, 16) |y| {
            for uint::range(0, 16) |x| {
                for uint::range(0, 16) |z| {
                    if !f((x,y,z), &self.blocks[y*16*16+z*16+x]) {
                        return
                    }
                }
            }
        }
    }

    fn each_block_mut(&mut self, f: &fn(pos: (uint, uint, uint), block: &mut Block) -> bool) {
        for uint::range(0, 16) |y| {
            for uint::range(0, 16) |x| {
                for uint::range(0, 16) |z| {
                    if !f((x,y,z), &mut self.blocks[y*16*16+z*16+x]) {
                        return
                    }
                }
            }
        }
    }

    fn generate_buffer_data(&self) -> (~[Vec3f], ~[Vec3f], ~[Vec3f]) {
        let mut vbuf = ~[];
        let mut tbuf = ~[];
        let mut nbuf = ~[];

        for self.each_block |(x,y,z), &block| {
            match block {
                Air => loop,
                _ => ()
            }
            vbuf.push_all_move(make_cube(x as float,y as float,z as float,0.5));
            tbuf.push_all_move(make_cube_texcoord());
            nbuf.push_all_move(make_cube_normal());
        }

        (vbuf, tbuf, nbuf)
    }

    fn update_buffer_cache(&mut self) {
        if self.buffer_cache.is_none() {
            self.buffer_cache = Some(BufferCache {
                position: Buffer::new(), texcoord: Buffer::new(), normal: Buffer::new(),
                vertex_no: 0
            });
        }

        let (v, t, n) = self.generate_buffer_data();
        match self.buffer_cache {
            Some(ref mut cache) => {
                cache.position.update(v);
                cache.texcoord.update(t);
                cache.normal.update(n);
                cache.vertex_no = v.len();
            },
            None => fail!(~"uninitialized buffer cache when populating")
        }
    }

    fn draw_cached(&self, program: &mut Program) {
        program.bind();

        let vertex_no;

        match self.buffer_cache {
            Some(ref cache) => {
                program.set_attribute_vec3("position", &cache.position);
                program.set_attribute_vec3("normal",   &cache.normal);
                program.set_attribute_vec3("texcoord", &cache.texcoord);
                vertex_no = cache.vertex_no;
            },
            None => fail!(~"uninitialized buffer cache when drawing")
        }

        glDrawArrays(GL_QUADS, 0, vertex_no as i32);
    }
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

fn make_cube_texcoord() -> ~[Vec3f] {
    ~[
        BaseVec3::new(0.0, 0.0, 1.0), BaseVec3::new(0.0, 1.0, 1.0), BaseVec3::new(1.0, 1.0, 1.0), BaseVec3::new(1.0, 0.0, 1.0),
        BaseVec3::new(0.0, 0.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(1.0, 1.0, 0.0), BaseVec3::new(1.0, 0.0, 0.0),
        BaseVec3::new(0.0, 0.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(1.0, 1.0, 0.0), BaseVec3::new(1.0, 0.0, 0.0),
        BaseVec3::new(0.0, 0.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(1.0, 1.0, 0.0), BaseVec3::new(1.0, 0.0, 0.0),
        BaseVec3::new(0.0, 0.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(1.0, 1.0, 0.0), BaseVec3::new(1.0, 0.0, 0.0),
        BaseVec3::new(0.0, 0.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(1.0, 1.0, 0.0), BaseVec3::new(1.0, 0.0, 0.0),
    ]
}

fn make_cube_normal() -> ~[Vec3f] {
    ~[
        BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0), BaseVec3::new(0.0, 1.0, 0.0),
        BaseVec3::new(0.0,-1.0, 0.0), BaseVec3::new(0.0,-1.0, 0.0), BaseVec3::new(0.0,-1.0, 0.0), BaseVec3::new(0.0,-1.0, 0.0),
        BaseVec3::new(-1.0,0.0, 0.0), BaseVec3::new(-1.0,0.0, 0.0), BaseVec3::new(-1.0,0.0, 0.0), BaseVec3::new(-1.0,0.0, 0.0),
        BaseVec3::new( 1.0,0.0, 0.0), BaseVec3::new( 1.0,0.0, 0.0), BaseVec3::new( 1.0,0.0, 0.0), BaseVec3::new( 1.0,0.0, 0.0),
        BaseVec3::new(0.0,0.0,  1.0), BaseVec3::new(0.0,0.0,  1.0), BaseVec3::new(0.0,0.0,  1.0), BaseVec3::new(0.0,0.0,  1.0),
        BaseVec3::new(0.0,0.0, -1.0), BaseVec3::new(0.0,0.0, -1.0), BaseVec3::new(0.0,0.0, -1.0), BaseVec3::new(0.0,0.0, -1.0),
    ]
}
