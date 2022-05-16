#version 300 es
layout (location = 0) in vec2 ModelPosition;
layout (location = 1) in vec2 aTexCoord;

out mediump vec4 VertexColor;
out mediump vec2 TexCoord;

uniform vec2 WorldPosition;
uniform vec2 Scale;
uniform vec4 Color;

void main() {
    gl_Position = vec4((ModelPosition * Scale) + WorldPosition, 0.0, 1.0);
    TexCoord = aTexCoord;
    VertexColor = Color;
}