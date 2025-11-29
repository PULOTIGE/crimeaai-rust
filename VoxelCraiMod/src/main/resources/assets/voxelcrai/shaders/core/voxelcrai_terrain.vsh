#version 330 core
// 🚀 VoxelCrai SH Lighting - Terrain Vertex Shader
// Воксельное освещение на основе Spherical Harmonics
// Портировано из Rust прототипа

// 📥 Входные атрибуты
in vec3 vaPosition;
in vec2 vaUV0;
in vec3 vaNormal;
in vec4 vaColor;
in ivec2 vaUV2;  // lightmap coords

// 📤 Выходные данные для фрагментного шейдера
out vec2 texCoord;
out vec3 worldPos;
out vec3 worldNormal;
out vec4 vertexColor;
out vec2 lightmapCoord;
out float depth;
out vec3 viewPos;

// 🌍 Униформы
uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;
uniform mat4 gbufferModelViewInverse;
uniform vec3 cameraPosition;

void main() {
    // 📍 Позиция в view space
    vec4 viewPosition = modelViewMatrix * vec4(vaPosition, 1.0);
    viewPos = viewPosition.xyz;
    
    // 📍 Позиция в мире
    worldPos = (gbufferModelViewInverse * viewPosition).xyz + cameraPosition;
    
    // 📐 Нормаль в мировых координатах
    worldNormal = normalize(mat3(gbufferModelViewInverse) * vaNormal);
    
    // 📝 Передача данных
    texCoord = vaUV0;
    vertexColor = vaColor;
    lightmapCoord = vaUV2 / 256.0;  // Нормализация lightmap
    
    // 🎯 Финальная позиция
    gl_Position = projectionMatrix * viewPosition;
    depth = gl_Position.z / gl_Position.w;
}
