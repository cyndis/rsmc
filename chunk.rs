use common::*;
use glcore::*;
use buffer::Buffer;
use shader::Program;

#[deriving(Eq)]
pub enum Block {
    Air,
    Grass,
    Stone,
    Dirt
}

pub impl Block {
    fn blocks(&self) -> bool {
        match *self {
            Air => false,
            Grass | Stone | Dirt => true
        }
    }

    fn top_texture_id(&self) -> uint {
        match *self {
            Air => 0,
            Grass => 0,
            Stone => 1,
            Dirt => 2
        }
    }

    fn side_texture_id(&self) -> uint {
        match *self {
            Air => 0,
            Grass => 3,
            Stone => 1,
            Dirt => 2
        }
    }

    fn breaking_time(&self) -> float {
        0.5
    }
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
            blocks: [Stone, ..16*16*16],
            buffer_cache: None
        }
    }

    fn block_at_vec(&self, pos: &Vec3f) -> Option<&'self Block> {
        let cc = (pos.x, pos.y, pos.z).floor();

        self.block_at(cc)
    }

    fn block_at(&self, cc: (int, int, int)) -> Option<&'self Block> {
        let (x, y, z) = cc;

        if x < 0 || x > 15 || y < 0 || y > 15 || z < 0 || z > 15 { return None }

        Some(&self.blocks[y*16*16+z*16+x])
    }

    fn block_at_mut(&mut self, cc: (int, int, int)) -> Option<&'self mut Block> {
        let (x, y, z) = cc;

        if x < 0 || x > 15 || y < 0 || y > 15 || z < 0 || z > 15 { return None }

        Some(&mut self.blocks[y*16*16+z*16+x])
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
            vbuf.push_all_move(make_cube(x as float+0.5,y as float+0.5,z as float+0.5,0.5));
            tbuf.push_all_move(make_cube_texcoord(block.top_texture_id(), block.side_texture_id()));
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

pub fn make_cube(x: float, y: float, z: float, n: float) -> ~[Vec3f] {
    ~[
        Vec3f::new(x-n,y+n,z-n), Vec3f::new(x-n,y+n,z+n), Vec3f::new(x+n,y+n,z+n), Vec3f::new(x+n,y+n,z-n),  // top
        Vec3f::new(x-n,y-n,z-n), Vec3f::new(x+n,y-n,z-n), Vec3f::new(x+n,y-n,z+n), Vec3f::new(x-n,y-n,z+n),  // bottom
        Vec3f::new(x-n,y-n,z-n), Vec3f::new(x-n,y-n,z+n), Vec3f::new(x-n,y+n,z+n), Vec3f::new(x-n,y+n,z-n),  // left
        Vec3f::new(x+n,y-n,z+n), Vec3f::new(x+n,y-n,z-n), Vec3f::new(x+n,y+n,z-n), Vec3f::new(x+n,y+n,z+n),  // right
        Vec3f::new(x-n,y-n,z+n), Vec3f::new(x+n,y-n,z+n), Vec3f::new(x+n,y+n,z+n), Vec3f::new(x-n,y+n,z+n),  // front
        Vec3f::new(x+n,y-n,z-n), Vec3f::new(x-n,y-n,z-n), Vec3f::new(x-n,y+n,z-n), Vec3f::new(x+n,y+n,z-n),  // back
    ]
}

pub fn make_cube_texcoord(tid: uint, tid2: uint) -> ~[Vec3f] {
    let tid = tid as float;
    let tid2 = tid2 as float;
    ~[
        Vec3f::new(0.0, 0.0, tid), Vec3f::new(0.0, 1.0, tid), Vec3f::new(1.0, 1.0, tid), Vec3f::new(1.0, 0.0, tid),
        Vec3f::new(0.0, 0.0, tid2), Vec3f::new(0.0, 1.0, tid2), Vec3f::new(1.0, 1.0, tid2), Vec3f::new(1.0, 0.0, tid2),
        Vec3f::new(0.0, 1.0, tid2), Vec3f::new(1.0, 1.0, tid2), Vec3f::new(1.0, 0.0, tid2), Vec3f::new(0.0, 0.0, tid2),
        Vec3f::new(0.0, 1.0, tid2), Vec3f::new(1.0, 1.0, tid2), Vec3f::new(1.0, 0.0, tid2), Vec3f::new(0.0, 0.0, tid2),
        Vec3f::new(0.0, 1.0, tid2), Vec3f::new(1.0, 1.0, tid2), Vec3f::new(1.0, 0.0, tid2), Vec3f::new(0.0, 0.0, tid2),
        Vec3f::new(0.0, 1.0, tid2), Vec3f::new(1.0, 1.0, tid2), Vec3f::new(1.0, 0.0, tid2), Vec3f::new(0.0, 0.0, tid2),
    ]
}

pub fn make_cube_normal() -> ~[Vec3f] {
    ~[
        Vec3f::new(0.0, 1.0, 0.0), Vec3f::new(0.0, 1.0, 0.0), Vec3f::new(0.0, 1.0, 0.0), Vec3f::new(0.0, 1.0, 0.0),
        Vec3f::new(0.0,-1.0, 0.0), Vec3f::new(0.0,-1.0, 0.0), Vec3f::new(0.0,-1.0, 0.0), Vec3f::new(0.0,-1.0, 0.0),
        Vec3f::new(-1.0,0.0, 0.0), Vec3f::new(-1.0,0.0, 0.0), Vec3f::new(-1.0,0.0, 0.0), Vec3f::new(-1.0,0.0, 0.0),
        Vec3f::new( 1.0,0.0, 0.0), Vec3f::new( 1.0,0.0, 0.0), Vec3f::new( 1.0,0.0, 0.0), Vec3f::new( 1.0,0.0, 0.0),
        Vec3f::new(0.0,0.0,  1.0), Vec3f::new(0.0,0.0,  1.0), Vec3f::new(0.0,0.0,  1.0), Vec3f::new(0.0,0.0,  1.0),
        Vec3f::new(0.0,0.0, -1.0), Vec3f::new(0.0,0.0, -1.0), Vec3f::new(0.0,0.0, -1.0), Vec3f::new(0.0,0.0, -1.0),
    ]
}
