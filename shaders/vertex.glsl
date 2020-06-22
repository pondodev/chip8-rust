#version 150 core

in vec2 a_Pos;
in vec3 a_Colour;

out vec4 v_Colour;

void main() {
    v_Colour = vec4(a_Colour, 1.0);
    gl_Position = vec4(a_Pos, 0.0, 1.0);
}