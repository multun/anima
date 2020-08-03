#version 450

layout(location = 0) in vec2 a_Pos;

layout(location = 0) out INTERFACE {
	vec2 uv; ///< UV coordinates.
} Out;

void main() {
    float u = (gl_VertexIndex << 1) & 2;
    float v = gl_VertexIndex & 2;
    Out.uv = vec2(u, 1. - v);
    gl_Position = vec4(vec2(u, v) * 2.0f + -1.0f, 0.0f, 1.0f);
}
