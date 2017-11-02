#version 150 core

uniform vec2 scale;

in vec2 in_position;
in vec3 in_color;
in float in_step;

out vec2 position;
out vec3 color;
out float step;

void main() {
    color = in_color;
    step = in_step;
    position = in_position;
    gl_Position = vec4(vec2(-1.0, 1.0) + in_position*scale, 0.0, 1.0);
}
