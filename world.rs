use chunk;
use chunk::Chunk;

use noise::Noise2DContext;

use shader::Program;

use core::hashmap::HashMap;

use common::*;

use lmath::vec::*;

pub struct World {
    loaded_chunks: HashMap<(int, int, int), Chunk>
}

fn new_test_chunk() -> Chunk {
    let mut c = Chunk::new();
    for c.each_block_mut |(x,y,z), block| {
        *block = if y == 0 { chunk::Stone } else { chunk::Air };
    };
    c.update_buffer_cache();
    c
}

fn new_stair_chunk() -> Chunk {
    let mut c = Chunk::new();
    for c.each_block_mut |(x,y,z), block| {
        if 16-x == y { *block = chunk::Stone } else { *block = chunk::Air };
        if x == 0 { *block = chunk::Stone };
    };
    c.update_buffer_cache();
    c
}

fn new_empty_chunk() -> Chunk {
    let mut c = Chunk::new();
    for c.each_block_mut |(x,y,z), block| {
        *block = chunk::Air;
    };
    c.update_buffer_cache();
    c
}

fn new_noise_chunk(x_offs: float, z_offs: float) -> Chunk {
    let mut c = Chunk::new();
    let ctx = Noise2DContext::new();
    for c.each_block_mut |(x,y,z), block| {
        let h = ctx.get(x_offs as f32 + x as f32 * 0.1, z_offs as f32 + z as f32 * 0.1);
        *block = if y as f32 / 6.0 < h {
            if y < 5 { chunk::Grass } else { chunk::Stone }
        } else { chunk::Air };
        if y == 0 { *block = chunk::Dirt }
    };
    c.update_buffer_cache();
    c
}

fn rem(a: int, b: int) -> int {
    if a >= 0 {
        a % b
    } else {
        b - (-a) % b
    }
}

fn div(a: int, b: int) -> int {
    if a >= 0 {
        a / b
    } else {
        a / b - 1
    }
}

pub impl World {
    fn new() -> World {
        let mut w = World {
            loaded_chunks: HashMap::new()
        };
        w.loaded_chunks.insert(( 0, 0, 0), new_empty_chunk());
        w.loaded_chunks.insert(( 0,-1, 0), new_noise_chunk( 0.0, 0.0));
        w.loaded_chunks.insert(( 1,-1, 0), new_noise_chunk( 1.6, 0.0));
        w.loaded_chunks.insert((-1,-1, 0), new_noise_chunk(-1.6, 0.0));
        w.loaded_chunks.insert(( 0,-1, 1), new_noise_chunk( 0.0, 1.6));
        w.loaded_chunks.insert(( 0,-1,-1), new_noise_chunk( 0.0,-1.6));
        w.loaded_chunks.insert(( 1,-1, 1), new_noise_chunk( 1.6, 1.6));
        w.loaded_chunks.insert((-1,-1, 1), new_noise_chunk(-1.6, 1.6));
        w.loaded_chunks.insert(( 1,-1,-1), new_noise_chunk( 1.6,-1.6));
        w.loaded_chunks.insert((-1,-1,-1), new_noise_chunk(-1.6,-1.6));
/*        w.loaded_chunks.insert((2,-1, 0), new_test_chunk());
        w.loaded_chunks.insert((1,-1, 0), new_stair_chunk());
        w.loaded_chunks.insert((1, 0, 0), new_empty_chunk());
        w.loaded_chunks.insert((2, 0, 0), new_empty_chunk());*/
        w
    }

    fn block_at_vec(&self, pos: &Vec3f) -> Option<&'self chunk::Block> {
        let cc = (pos.x,pos.y,pos.z).floor();

        self.block_at(cc)
    }

    fn block_at(&self, cc: (int, int, int)) -> Option<&'self chunk::Block> {
        let (x, y, z) = cc;

        match self.loaded_chunks.find(&(div(x,16), div(y,16), div(z,16))) {
            Some(ref chunk) => chunk.block_at((rem(x,16), rem(y,16), rem(z,16))),
            None => None
        }
    }

    fn each_chunk(&self, f: &fn(&(int,int,int), &'self Chunk) -> bool) {
        for self.loaded_chunks.each |cc, chunk| {
            if !f(cc, chunk) { return }
        }
    }

    fn cast_ray(&self, origin: &Vec3f, direction: &Vec3f) ->
        Option<((int, int, int), &'self chunk::Block)>
    {
        // stupid algorithm
        let d = direction.normalize().mul_t(0.1);

        for uint::range(0, 8*8) |i| {
            let pos = origin.add_v(&d.mul_t(i as float));

            let block = self.block_at_vec(&pos);
            match block {
                Some(&chunk::Air) | None => (),
                Some(b) => return Some(((pos.x, pos.y, pos.z).floor(), b))
            };
        }

        None
    }

    fn cast_ray_previous(&self, origin: &Vec3f, direction: &Vec3f) ->
        Option<((int, int, int), &'self chunk::Block)>
    {
        // stupid algorithm
        let d = direction.normalize().mul_t(0.1);

        let mut prev = None;

        for uint::range(0, 8*8) |i| {
            let pos = origin.add_v(&d.mul_t(i as float));

            let block = self.block_at_vec(&pos);
            match block {
                None => (),
                Some(b) if *b == chunk::Air => prev = Some(((pos.x, pos.y, pos.z).floor(), b)),
                Some(b) => return prev
            };
        }

        None
    }

    fn replace_block(&mut self, cc: (int, int, int), new_block: chunk::Block) {
        let (x, y, z) = cc;

        match self.loaded_chunks.find_mut(&(div(x,16), div(y,16), div(z, 16))) {
            Some(ref chunk) => {
                let mut block = chunk.block_at_mut((rem(x,16), rem(y,16), rem(z, 16))).unwrap();
                *block = new_block;
                chunk.update_buffer_cache();
            },
            None => fail!(~"replace_block in unloaded chunk")
        }
    }
}
