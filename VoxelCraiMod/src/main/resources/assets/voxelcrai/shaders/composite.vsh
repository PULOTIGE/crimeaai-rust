#version 330 core
// 🚀 VoxelCrai - Composite Vertex Shader
// Fullscreen quad для пост-обработки

out vec2 texCoord;

void main() {
    // 📐 Fullscreen quad через gl_VertexID
    // 0: (-1, -1), 1: (1, -1), 2: (-1, 1), 3: (1, 1)
    vec2 pos = vec2(
        float((gl_VertexID & 1) * 2 - 1),
        float((gl_VertexID >> 1) * 2 - 1)
    );
    
    texCoord = pos * 0.5 + 0.5;
    gl_Position = vec4(pos, 0.0, 1.0);
}
