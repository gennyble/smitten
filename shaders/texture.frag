#version 300 es
out mediump vec4 FragColor;

in mediump vec2 TexCoord;
in mediump vec4 VertexColor;

uniform sampler2D Texture;

void main() {
    FragColor = VertexColor + texture(Texture, TexCoord);
}