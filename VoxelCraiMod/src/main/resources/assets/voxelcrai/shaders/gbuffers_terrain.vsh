#version 330 core
// 🚀 VoxelCrai SH Lighting - gbuffers_terrain Vertex Shader
// Стандартное имя для Iris/OptiFine совместимости

// 📥 Входные атрибуты (стандартные Minecraft)
in vec3 vaPosition;
in vec2 vaUV0;
in vec3 vaNormal;
in vec4 vaColor;
in ivec2 vaUV2;

// 📤 Выходные данные
out vec2 texCoord;
out vec3 worldPos;
out vec3 worldNormal;
out vec4 vertexColor;
out vec2 lightmapCoord;
out float viewDist;

// 🌍 Униформы
uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;
uniform mat4 gbufferModelViewInverse;
uniform vec3 cameraPosition;

void main() {
    // 📍 View space position
    vec4 viewPos = modelViewMatrix * vec4(vaPosition, 1.0);
    
    // 📍 World position
    worldPos = (gbufferModelViewInverse * viewPos).xyz + cameraPosition;
    
    // 📐 World normal
    worldNormal = normalize(mat3(gbufferModelViewInverse) * vaNormal);
    
    // 📝 Pass-through data
    texCoord = vaUV0;
    vertexColor = vaColor;
    lightmapCoord = vaUV2 / 256.0;
    
    // 📏 View distance (for fog/LOD)
    viewDist = length(viewPos.xyz);
    
    // 🎯 Clip position
    gl_Position = projectionMatrix * viewPos;
}
