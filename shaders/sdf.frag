#version 410
precision mediump float;

in vec4 vertexColor;
in vec4 vertexPointPair;
in vec4 vertexParameters;
flat in int vertexDrawMethod;

out vec4 FragColor;

void main() {
	if (vertexDrawMethod == 1) {
		vec2 center = gl_FragCoord.xy - vertexPointPair.xy;
		float dist = length(center) - vertexPointPair.z;

		if (dist <= 0.0) {
			FragColor = vertexColor;
		} else {
			FragColor = vec4(1.0, 0.0, 0.0, 0.0);
		}
	} else if (vertexDrawMethod == 2) {
		vec2 p1 = vertexPointPair.xy;
		vec2 p2 = vertexPointPair.zw;

		vec2 point = gl_FragCoord.xy;

		vec2 pa = point - p1, ba = p2 - p1;
		float h = clamp( dot(pa,ba) / dot(ba,ba), 0.0, 1.0 );
		float dist = length(pa - ba * h);

		if (dist <= vertexParameters.x) {
			FragColor = vertexColor;
		} else {
			FragColor = vec4(0.0, 1.0, 0.0, 0.0);
		}
	} else {
		FragColor = vec4(0.0, 0.0, 1.0, 1.0);
	}
}
