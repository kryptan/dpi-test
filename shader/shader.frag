#version 150 core

in vec2 position;
in vec3 color;
in float step;

out vec4 pixel;

void main() {
    bool a = mod(position.x, 2.0*step) > step;
    bool b = mod(position.y, 2.0*step) > step;
    float k = 1.0 - float(int(a)^int(b));
    pixel = vec4(color.rgb, k);
}
