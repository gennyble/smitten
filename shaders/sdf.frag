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
		} else if (dist <= 2.0) {
			FragColor = vec4(0.0, 0.0, 0.0, 1.0);
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
	} else if (vertexDrawMethod == 3) {
		const vec3 k = vec3(-0.9238795325, 0.3826834323, 0.4142135623 );
		float r = 5;
		vec2 p = vertexPointPair.xy;
		p = abs(p);
		p -= 2.0*min(dot(vec2( k.x,k.y),p),0.0)*vec2( k.x,k.y);
		p -= 2.0*min(dot(vec2(-k.x,k.y),p),0.0)*vec2(-k.x,k.y);
		p -= vec2(clamp(p.x, -k.z*r, k.z*r), r);
		float dist = length(p)*sign(p.y);

		if (dist <= 0.0) {
			FragColor = vertexColor;
		} else {
			FragColor = vec4(0.0, 0.0, 0.0, 0.0);
		}
	} else {
		FragColor = vec4(0.0, 0.0, 1.0, 1.0);
	}
}
