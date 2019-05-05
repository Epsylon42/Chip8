#version 150

in vec2 pos;
out vec2 tex_pos;

void main() {
    gl_Position = vec4(pos, 0, 1);
    tex_pos = pos / 2.0 + vec2(0.5);
}
