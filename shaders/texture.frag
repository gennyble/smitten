#version 410
out mediump vec4 FragColor;

in mediump vec2 TexCoord;
in mediump vec4 VertexColor;
flat in int ColorTex;

uniform sampler2D Texture;

void main() {
    if (ColorTex == 1) {
        //FragColor = vec4(0.0, 0.0, 1.0, 1.0);
        FragColor = VertexColor + texture(Texture, TexCoord);
    } else if (ColorTex == 2) {//FragColor = vec4(0.0, 1.0, 0.0, 1.0);
        FragColor = VertexColor;
    } else {
        //ragColor = vec4(1.0, 0.0, 0.0, 1.0);
        FragColor = texture(Texture, TexCoord);
    }
}