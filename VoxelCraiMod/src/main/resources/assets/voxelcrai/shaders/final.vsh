#version 330 core
// 🚀 VoxelCrai - Final Pass Vertex Shader

out vec2 texCoord;

void main() {
    vec2 pos = vec2(
        float((gl_VertexID & 1) * 2 - 1),
        float((gl_VertexID >> 1) * 2 - 1)
    );
    
    texCoord = pos * 0.5 + 0.5;
    gl_Position = vec4(pos, 0.0, 1.0);
}
