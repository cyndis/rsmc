use stb_image::image::*;
use glcore::*;

pub struct Texture {
    handle: u32
}

impl Drop for Texture {
    fn finalize(&self) {
        glDeleteTextures(1, unsafe { ptr::addr_of(&self.handle) });
    }
}

// glcore doesn't include EXT_direct_state_access
extern {
    fn glTextureParameteriEXT(texture: GLuint, target: GLenum, pname: GLenum, param: GLuint);
    fn glTextureImage2DEXT(texture: GLuint, target: GLenum, level: GLint,
                           internalformat: GLenum, width: GLsizei, height: GLsizei,
                           border: GLint, format: GLenum, typ: GLenum, pixels: *GLvoid);
    fn glBindMultiTextureEXT(unit: GLenum, target: GLenum, texture: GLuint);
}

pub impl Texture {
    fn load_file(name: ~str) -> Option<Texture> {
        match load_with_depth(name, 3, false) {
            ImageU8(image) => {
                let tex = 0u32;

                glGenTextures(1, unsafe { ptr::addr_of(&tex) });

                unsafe {
                glTextureParameteriEXT(tex, GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
                glTextureParameteriEXT(tex, GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
                glTextureParameteriEXT(tex, GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);

                glTextureImage2DEXT(tex, GL_TEXTURE_2D, 0, GL_RGB, image.width as GLsizei,
                                    image.height as GLsizei, 0, GL_RGB, GL_UNSIGNED_BYTE,
                                    unsafe{ cast::transmute(&image.data[0]) });
                }

                Some(Texture { handle: tex })
            },
            _ => None
        }
    }

    fn bind(&self, unit: u32) {
        unsafe {
            glBindMultiTextureEXT(GL_TEXTURE0+unit, GL_TEXTURE_2D, self.handle);
        }
    }
}
