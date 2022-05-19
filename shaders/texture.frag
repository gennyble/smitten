#version 300 es
out mediump vec4 FragColor;

in mediump vec2 TexCoord;
in mediump vec4 VertexColor;
flat in int ColorTex;

uniform sampler2D Texture;

void main() {
    if (ColorTex == 1) {
        FragColor = VertexColor + texture(Texture, TexCoord);
    } else if (ColorTex == 2) {
        FragColor = VertexColor;
    } else {
        FragColor = texture(Texture, TexCoord);
    }
}