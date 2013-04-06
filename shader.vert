#version 330
in vec3 position;
in vec3 texcoord;
in vec3 normal;
uniform mat4 projection;
uniform mat4 modelview;

out vec3 v_texcoord;
out vec3 v_position;

out vec4 lieye;
out vec4 vneye;

void main() {
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_texcoord = texcoord;
    v_position = position;

    lieye = modelview * vec4(2.0, 1.0, 1.0, 0.0);
    vneye = modelview * vec4(normal, 0.0);
}
