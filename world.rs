use chunk;
use chunk::Chunk;

use shader::Program;

use core::hashmap::HashMap;

use common::*;

pub struct World {
    loaded_chunks: HashMap<(int, int, int), Chunk>
}

fn new_test_chunk() -> Chunk {
    let mut c = Chunk::new();
    for c.each_block_mut |(x,y,z), block| {
        *block = if y == 0 { chunk::Brick } else { chunk::Air };
    };
    c.update_buffer_cache();
    c
}

fn new_stair_chunk() -> Chunk {
    let mut c = Chunk::new();
    for c.each_block_mut |(x,y,z), block| {
        if 16-x == y { *block = chunk::Brick } else { *block = chunk::Air };
        if x == 0 { *block = chunk::Brick };
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
        w.loaded_chunks.insert((0, 0, 0), new_test_chunk());
        w.loaded_chunks.insert((2,-1, 0), new_test_chunk());
        w.loaded_chunks.insert((1,-1, 0), new_stair_chunk());
        w.loaded_chunks.insert((1, 0, 0), new_empty_chunk());
        w.loaded_chunks.insert((2, 0, 0), new_empty_chunk());
        w
    }

    fn block_at(&self, pos: &Vec3f) -> Option<&'self chunk::Block> {
        let (x, y, z) = (pos.x,pos.y,pos.z).floor();

        /*
        io::println(fmt!("(%?,%?,%?) => chunk (%?,%?,%?) block (%?,%?,%?)",
                         pos.x, pos.y, pos.z, div(x, 16), div(y, 16), div(z, 16),
                         rem(x, 16), rem(y, 16), rem(z, 16)
                        ));
                        */

        match self.loaded_chunks.find(&(div(x,16), div(y,16), div(z,16))) {
            Some(ref chunk) => chunk.block_at((rem(x,16), rem(y,16), rem(z,16))),
            None => None
        }
    }

    fn each_chunk(&self, f: &fn(&(int,int,int), &'self Chunk) -> bool) {
        for self.loaded_chunks.each |&(cc, chunk)| {
            if !f(cc, chunk) { return }
        }
    }
}
