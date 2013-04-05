use glcore::*;

pub struct Buffer {
    handle: u32
}

pub impl Buffer {
    fn new() -> Buffer {
        let buf = 0u32;
        glGenBuffers(1, unsafe { ptr::addr_of(&buf) });

        Buffer {
            handle: buf
        }
    }

    fn bind(&self) {
        glBindBuffer(GL_ARRAY_BUFFER, self.handle);
    }

    fn update<T>(&self, data: &[T]) {
        self.bind();

        do vec::as_imm_buf(data) |ptr, len| {
            glBufferData(GL_ARRAY_BUFFER, (len * sys::size_of::<T>()) as i64,
                         ptr as *libc::c_void, GL_STATIC_DRAW);
        }
    }
}
