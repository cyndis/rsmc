use stb_image::image::*;
use glcore::*;

pub struct Texture {
    handle: u32,
    target: GLenum
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
    fn glTextureImage3DEXT(texture: GLuint, target: GLenum, level: GLint,
                           internalformat: GLenum, width: GLsizei, height: GLsizei,
                           depth: GLsizei, border: GLint, format: GLenum, typ: GLenum,
                           pixels: *GLvoid);
    fn glBindMultiTextureEXT(unit: GLenum, target: GLenum, texture: GLuint);
}

pub enum TextureFormat {
    SingleTexture,
    TextureArray(uint)
}

pub impl Texture {
    fn load_file(name: ~str, format: TextureFormat) -> Option<Texture> {
        match load_with_depth(name, 3, false) {
            ImageU8(image) => {
                let tex = 0u32;

                glGenTextures(1, unsafe { ptr::addr_of(&tex) });

                let target = match format {
                    SingleTexture => GL_TEXTURE_2D,
                    TextureArray(*) => GL_TEXTURE_2D_ARRAY
                };

                unsafe {
                glTextureParameteriEXT(tex, target, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
                glTextureParameteriEXT(tex, target, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
                glTextureParameteriEXT(tex, target, GL_TEXTURE_WRAP_S, GL_REPEAT);
                glTextureParameteriEXT(tex, target, GL_TEXTURE_WRAP_T, GL_REPEAT);
                // GL_MAX_ANISOTROPY_EXT
                glTextureParameteriEXT(tex, target, 0x84FE, 16);

                match format {
                    SingleTexture => glTextureImage2DEXT(tex, GL_TEXTURE_2D, 0, GL_RGB,
                                                         image.width as GLsizei,
                                                         image.height as GLsizei, 0, GL_RGB,
                                                         GL_UNSIGNED_BYTE,
                                                         cast::transmute(&image.data[0])),
                    TextureArray(n) => glTextureImage3DEXT(tex, GL_TEXTURE_2D_ARRAY, 0,
                                                           GL_RGB, image.width as GLsizei,
                                                           (image.height / n) as GLsizei,
                                                           n as GLsizei, 0, GL_RGB, GL_UNSIGNED_BYTE,
                                                           cast::transmute(&image.data[0]))
                }
                }

                Some(Texture { handle: tex, target: target })
            },
            _ => None
        }
    }

    fn bind(&self, unit: u32) {
        unsafe {
            glBindMultiTextureEXT(GL_TEXTURE0+unit, self.target, self.handle);
        }
    }
}
