use glcore::*;
use common::*;
use buffer::Buffer;

pub struct Program {
    handle: u32,
    vs: u32,
    fs: u32
}

pub impl Program {
    fn new(vs_src: &str, fs_src: &str) -> Program {
        let program = glCreateProgram();

        let vs = glCreateShader(GL_VERTEX_SHADER);
        match load_shader_source(vs, vs_src) {
            Ok(*) => (),
            Err(error) => fail!(error)
        };

        let fs = glCreateShader(GL_FRAGMENT_SHADER);
        match load_shader_source(fs, fs_src) {
            Ok(*) => (),
            Err(error) => fail!(error)
        };

        glAttachShader(program, vs);
        glAttachShader(program, fs);

        glLinkProgram(program);

        Program {
            handle: program,
            vs: vs,
            fs: fs
        }
    }

    fn attribute_location(&self, attrib: &str) -> u32 {
        do str::as_c_str(attrib) |ptr| {
            glGetAttribLocation(self.handle, ptr) as u32
        }
    }

    fn set_attribute_vec3(&mut self, attrib: &str, buffer: &Buffer) {
        glUseProgram(self.handle);
        buffer.bind();
        glVertexAttribPointer(self.attribute_location(attrib), 3, GL_DOUBLE,
                              GL_FALSE, sys::size_of::<Vec3f>() as i32, 0 as *libc::c_void);
        glEnableVertexAttribArray(self.attribute_location(attrib));
    }

    fn bind(&self) {
        glUseProgram(self.handle);
    }
}

fn load_shader_source(handle: u32, source: &str) -> Result<(), ~str> {
    do str::as_buf(source) |ptr, len| {
        unsafe {
            let l = len as i32;
            glShaderSource(handle, 1, cast::transmute(ptr::addr_of(&ptr)), ptr::addr_of(&l));
        }
    }
    glCompileShader(handle);

    let success: i32 = 0;
    glGetShaderiv(handle, GL_COMPILE_STATUS, unsafe { ptr::addr_of(&success) });
    if success as u8 == GL_TRUE { return Ok(()) }

    let info_len: i32 = 0;

    glGetShaderiv(handle, GL_INFO_LOG_LENGTH, unsafe { ptr::addr_of(&info_len) });

    let mut info_log: ~[u8] = vec::from_elem(info_len as uint, 0);

    do vec::as_mut_buf(info_log) |ptr, len| {
        glGetShaderInfoLog(handle, len as i32, ptr::null(), unsafe { cast::transmute(ptr) })
    }

    Err(str::from_bytes(info_log))
}
