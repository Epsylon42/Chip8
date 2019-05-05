#version 150

in vec2 tex_pos;
uniform sampler2D tex;

void main() {
    gl_FragColor = vec4(vec3(texture(tex, vec2(tex_pos.x, 1.0 - tex_pos.y)).x), 1);
}
