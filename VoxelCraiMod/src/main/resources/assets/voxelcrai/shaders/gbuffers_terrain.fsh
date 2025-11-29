#version 430 core
// 🚀 VoxelCrai SH Lighting - gbuffers_terrain Fragment Shader
// Основной шейдер террейна с SH освещением

// 📥 Входные данные
in vec2 texCoord;
in vec3 worldPos;
in vec3 worldNormal;
in vec4 vertexColor;
in vec2 lightmapCoord;
in float viewDist;

// 📤 Выходные буферы
/* DRAWBUFFERS:0123 */
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outNormal;
layout(location = 2) out vec4 outSpecular;
layout(location = 3) out vec4 outLighting;

// 🖼️ Текстуры
uniform sampler2D gtexture;
uniform sampler2D lightmap;
uniform sampler2D normals;    // Normal map (если есть)
uniform sampler2D specular;   // Specular map (если есть)

// 🌍 Униформы
uniform vec3 cameraPosition;
uniform vec3 sunPosition;
uniform float rainStrength;
uniform int worldTime;
uniform float near;
uniform float far;

// ============================================
// 🔮 SH Lighting System
// ============================================

// SH константы (Band 0-2)
const float SH_C0 = 0.282095;
const float SH_C1 = 0.488603;
const float SH_C2_0 = 1.092548;
const float SH_C2_1 = 0.315392;
const float SH_C2_2 = 0.546274;

// Косинусная свертка
const float CL0 = 3.14159265;
const float CL1 = 2.09439510;
const float CL2 = 0.78539816;

// 📦 SSBO паттернов
layout(std430, binding = 0) buffer PatternBuffer {
    uint patternData[];
};
uniform int activePatternCount;

// 🔑 Пространственный хеш
int getPatternIdx(vec3 pos) {
    ivec3 cell = ivec3(floor(pos / 4.0));
    int hash = cell.x * 73856093 ^ cell.y * 19349663 ^ cell.z * 83492791;
    return abs(hash) % max(1, activePatternCount);
}

// 🔮 SH базис для направления
void shBasis(vec3 d, out float b[9]) {
    b[0] = SH_C0;
    b[1] = SH_C1 * d.y;
    b[2] = SH_C1 * d.z;
    b[3] = SH_C1 * d.x;
    b[4] = SH_C2_0 * d.x * d.y;
    b[5] = SH_C2_0 * d.y * d.z;
    b[6] = SH_C2_1 * (3.0 * d.z * d.z - 1.0);
    b[7] = SH_C2_0 * d.x * d.z;
    b[8] = SH_C2_2 * (d.x * d.x - d.y * d.y);
}

// 🔮 Получение SH коэффициентов из буфера
void getSHCoeffs(int idx, out float c[9]) {
    // Дефолты
    c[0] = 0.5;
    for (int i = 1; i < 9; i++) c[i] = 0.0;
    c[2] = 0.3; // Верхний свет
    
    if (idx >= 0 && idx < activePatternCount) {
        int off = idx * 256 + 8; // Skip id + padding + lights
        for (int i = 0; i < 9; i += 4) {
            uint packed = patternData[off + i/4];
            c[i] = float(int(packed & 0xFFu) - 128) / 127.0;
            if (i+1 < 9) c[i+1] = float(int((packed >> 8) & 0xFFu) - 128) / 127.0;
            if (i+2 < 9) c[i+2] = float(int((packed >> 16) & 0xFFu) - 128) / 127.0;
            if (i+3 < 9) c[i+3] = float(int((packed >> 24) & 0xFFu) - 128) / 127.0;
        }
    }
}

// 🔮 Вычисление SH освещения
vec3 evalSH(float c[9], vec3 n) {
    float b[9];
    shBasis(n, b);
    
    float irr = c[0] * b[0] * CL0;
    irr += (c[1] * b[1] + c[2] * b[2] + c[3] * b[3]) * CL1;
    irr += (c[4] * b[4] + c[5] * b[5] + c[6] * b[6] + c[7] * b[7] + c[8] * b[8]) * CL2;
    
    return vec3(max(0.0, irr));
}

// 🌑 SH тень
float shShadow(float c[9], vec3 lightDir) {
    float b[9];
    shBasis(lightDir, b);
    
    float vis = 0.0;
    for (int i = 0; i < 9; i++) vis += c[i] * b[i];
    
    return clamp(vis * 0.5 + 0.5, 0.0, 1.0);
}

// 🎨 Определение материала
void getMat(vec3 col, out float rough, out float metal, out float emiss) {
    rough = 0.7;
    metal = 0.0;
    emiss = 0.0;
    
    float sat = max(max(col.r, col.g), col.b) - min(min(col.r, col.g), col.b);
    
    // Металл
    if (col.r > 0.6 && col.g > 0.4 && col.g < 0.8 && col.b < 0.4) {
        rough = 0.3;
        metal = 0.9;
    }
    
    // Вода
    if (col.b > 0.6 && col.r < 0.3) rough = 0.05;
    
    // Лава
    if (col.r > 0.9 && col.g < 0.6) emiss = 1.0;
}

// ============================================
// 🎯 Main
// ============================================

void main() {
    // 🎨 Albedo
    vec4 albedo = texture(gtexture, texCoord) * vertexColor;
    if (albedo.a < 0.1) discard;
    
    // 📐 Normal
    vec3 N = normalize(worldNormal);
    
    // 🔧 Material
    float roughness, metallic, emission;
    getMat(albedo.rgb, roughness, metallic, emission);
    
    // 💡 Vanilla lightmap
    vec3 lm = texture(lightmap, lightmapCoord).rgb;
    
    // 🔮 SH lighting
    int pIdx = getPatternIdx(worldPos);
    float shC[9];
    getSHCoeffs(pIdx, shC);
    
    vec3 shLight = evalSH(shC, N);
    
    // ☀️ Sun direction & shadow
    vec3 sunDir = normalize(sunPosition);
    float dayFactor = max(0.0, sunDir.y);
    float shadow = shShadow(shC, sunDir);
    
    // 💡 Direct + indirect
    vec3 direct = vec3(0.9, 0.85, 0.8) * shadow * dayFactor * (1.0 - rainStrength * 0.5);
    vec3 indirect = shLight * 0.5;
    
    // 🎨 Final lighting
    vec3 lighting = direct + indirect;
    lighting = max(lighting, lm * 0.5);  // Blend with vanilla
    
    // 📦 Output
    outColor = vec4(albedo.rgb, 1.0);
    outNormal = vec4(N * 0.5 + 0.5, roughness);
    outSpecular = vec4(metallic, emission, 0.0, 1.0);
    outLighting = vec4(lighting, 1.0);
}
