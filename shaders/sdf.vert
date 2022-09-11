#version 410
layout (location = 0) in vec2 ModelPosition;

uniform vec2 WorldPosition;
uniform vec2 Scale;
uniform vec4 Color;
uniform vec4 PointPair;
uniform vec4 Parameters;
uniform int DrawMethod;

out vec4 vertexColor;
out vec4 vertexPointPair;
out vec4 vertexParameters;
flat out int vertexDrawMethod;

void main() {
	vertexColor = Color;
	vertexPointPair = PointPair;
	vertexParameters = Parameters;
	vertexDrawMethod = DrawMethod;

	gl_Position = vec4((ModelPosition * Scale) + WorldPosition, 0.0, 1.0);
}