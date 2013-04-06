#version 330
#extension GL_EXT_texture_array : enable
layout (location = 0) out vec4 outputColor;
uniform sampler2DArray texture;

in vec3 v_texcoord;
in vec3 v_position;

in vec4 lieye;
in vec4 vneye;

void main() {
    vec4 Ld = texture2DArray(texture, v_texcoord);

    vec4 n_eye = normalize(vneye);

    vec4 Ia = vec4(0.13, 0.13, 0.13, 1.0);
    vec4 Id = vec4(0.75, 0.75, 0.75, 1.0) * max(dot(lieye, n_eye), 0.0);
    outputColor = Ld * (Ia + Id);
}
