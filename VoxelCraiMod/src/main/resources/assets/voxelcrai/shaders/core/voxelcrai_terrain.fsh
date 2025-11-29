#version 430 core
// 🚀 VoxelCrai SH Lighting - Terrain Fragment Shader
// Воксельное освещение с SH паттернами
// ТОЧКА ВХОДА для вычисления освещения блоков

// 📥 Входные данные из вершинного шейдера
in vec2 texCoord;
in vec3 worldPos;
in vec3 worldNormal;
in vec4 vertexColor;
in vec2 lightmapCoord;
in float depth;
in vec3 viewPos;

// 📤 Выходные буферы (G-buffer)
layout(location = 0) out vec4 outColor;      // colortex0: albedo + alpha
layout(location = 1) out vec4 outNormal;     // colortex1: normal + roughness
layout(location = 2) out vec4 outLightmap;   // colortex2: lightmap + metallic + emission
layout(location = 3) out vec4 outShLight;    // colortex3: SH lighting result

// 🖼️ Текстуры
uniform sampler2D gtexture;
uniform sampler2D lightmap;

// 🌍 Униформы
uniform vec3 cameraPosition;
uniform vec3 sunPosition;
uniform float rainStrength;
uniform int worldTime;

// ============================================
// 🔮 SH Constants and Functions
// ============================================

// 📐 SH константы
const float SH_C0 = 0.282095;
const float SH_C1 = 0.488603;
const float SH_C2_0 = 1.092548;
const float SH_C2_1 = 0.315392;
const float SH_C2_2 = 0.546274;

// 📐 Косинусная свертка для diffuse
const float COSINE_LOBE_0 = 3.14159265;
const float COSINE_LOBE_1 = 2.09439510;
const float COSINE_LOBE_2 = 0.78539816;

// 💡 Структура паттерна (упрощенная для встроенного использования)
struct LightPattern {
    vec3 directLight;
    vec3 indirectLight;
    float shCoeffs[9];
    float roughness;
    float metallic;
    float ao;
};

// 📦 SSBO с паттернами
layout(std430, binding = 0) buffer PatternBuffer {
    uint patternData[];
};

uniform int activePatternCount;

// 🔑 Пространственный хеш для индекса паттерна
int getPatternIndex(vec3 pos) {
    ivec3 cell = ivec3(floor(pos / 4.0));
    int hash = cell.x * 73856093 ^ cell.y * 19349663 ^ cell.z * 83492791;
    return abs(hash) % max(1, activePatternCount);
}

// 🔮 Вычисление SH базиса для направления (3 bands = 9 коэффициентов)
void computeSHBasis(vec3 dir, out float basis[9]) {
    float x = dir.x;
    float y = dir.y;
    float z = dir.z;
    
    // Band 0
    basis[0] = SH_C0;
    
    // Band 1
    basis[1] = SH_C1 * y;
    basis[2] = SH_C1 * z;
    basis[3] = SH_C1 * x;
    
    // Band 2
    basis[4] = SH_C2_0 * x * y;
    basis[5] = SH_C2_0 * y * z;
    basis[6] = SH_C2_1 * (3.0 * z * z - 1.0);
    basis[7] = SH_C2_0 * x * z;
    basis[8] = SH_C2_2 * (x * x - y * y);
}

// 🔮 Реконструкция diffuse освещения из SH
vec3 evaluateSHDiffuse(float coeffs[9], vec3 normal) {
    float basis[9];
    computeSHBasis(normal, basis);
    
    float irradiance = 0.0;
    
    // Band 0
    irradiance += coeffs[0] * basis[0] * COSINE_LOBE_0;
    
    // Band 1
    irradiance += (coeffs[1] * basis[1] + coeffs[2] * basis[2] + coeffs[3] * basis[3]) * COSINE_LOBE_1;
    
    // Band 2
    irradiance += (coeffs[4] * basis[4] + coeffs[5] * basis[5] + coeffs[6] * basis[6] +
                   coeffs[7] * basis[7] + coeffs[8] * basis[8]) * COSINE_LOBE_2;
    
    return vec3(max(0.0, irradiance));
}

// 🌑 Вычисление тени из SH
float computeSHShadow(float coeffs[9], vec3 lightDir) {
    float basis[9];
    computeSHBasis(lightDir, basis);
    
    float visibility = 0.0;
    for (int i = 0; i < 9; i++) {
        visibility += coeffs[i] * basis[i];
    }
    
    return clamp(visibility * 0.5 + 0.5, 0.0, 1.0);
}

// 📖 Чтение паттерна из SSBO
LightPattern fetchPattern(int idx) {
    LightPattern pattern;
    
    // Дефолтные значения
    pattern.directLight = vec3(0.8, 0.75, 0.7);
    pattern.indirectLight = vec3(0.2, 0.22, 0.25);
    pattern.roughness = 0.7;
    pattern.metallic = 0.0;
    pattern.ao = 1.0;
    
    // Дефолтные SH коэффициенты (ambient dome)
    pattern.shCoeffs[0] = 0.5;  // DC
    for (int i = 1; i < 9; i++) {
        pattern.shCoeffs[i] = 0.0;
    }
    // Небольшой верхний свет
    pattern.shCoeffs[2] = 0.3;  // Y10 (z direction)
    
    if (idx >= 0 && idx < activePatternCount) {
        // 📍 Смещение в буфере (256 uint на паттерн)
        int offset = idx * 256;
        
        // Пропускаем ID и padding
        offset += 4;
        
        // 💡 Direct light RGB (упакованы как fp16)
        uint packed0 = patternData[offset];
        uint packed1 = patternData[offset + 1];
        pattern.directLight.r = unpackHalf2x16(packed0).x;
        pattern.directLight.g = unpackHalf2x16(packed0).y;
        pattern.directLight.b = unpackHalf2x16(packed1).x;
        offset += 2;
        
        // 🌙 Indirect light
        packed0 = patternData[offset];
        packed1 = patternData[offset + 1];
        pattern.indirectLight.r = unpackHalf2x16(packed0).x;
        pattern.indirectLight.g = unpackHalf2x16(packed0).y;
        pattern.indirectLight.b = unpackHalf2x16(packed1).x;
        offset += 2;
        
        // 🔮 SH коэффициенты (первые 9, упакованы по 4 в uint)
        for (int i = 0; i < 9; i += 4) {
            uint packedSH = patternData[offset + i/4];
            pattern.shCoeffs[i+0] = float(int(packedSH & 0xFFu) - 128) / 127.0;
            if (i+1 < 9) pattern.shCoeffs[i+1] = float(int((packedSH >> 8) & 0xFFu) - 128) / 127.0;
            if (i+2 < 9) pattern.shCoeffs[i+2] = float(int((packedSH >> 16) & 0xFFu) - 128) / 127.0;
            if (i+3 < 9) pattern.shCoeffs[i+3] = float(int((packedSH >> 24) & 0xFFu) - 128) / 127.0;
        }
    }
    
    return pattern;
}

// 🎨 Определение материала по цвету
void getMaterial(vec3 color, out float roughness, out float metallic, out float emission) {
    roughness = 0.7;
    metallic = 0.0;
    emission = 0.0;
    
    float gray = (color.r + color.g + color.b) / 3.0;
    float sat = max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
    
    // 🪨 Камень
    if (sat < 0.1 && gray > 0.3 && gray < 0.7) {
        roughness = 0.9;
    }
    
    // ⛏️ Металл (золотистый)
    if (color.r > 0.6 && color.g > 0.4 && color.g < 0.8 && color.b < 0.4) {
        roughness = 0.3;
        metallic = 0.9;
    }
    
    // 💧 Вода
    if (color.b > 0.6 && color.g > 0.5 && color.r < 0.3) {
        roughness = 0.05;
    }
    
    // 🔥 Лава/огонь
    if (color.r > 0.9 && color.g > 0.3 && color.g < 0.7) {
        emission = 1.0;
    }
}

// ============================================
// 🎯 Main
// ============================================

void main() {
    // 🎨 Базовый цвет
    vec4 albedo = texture(gtexture, texCoord) * vertexColor;
    
    // ✂️ Alpha test
    if (albedo.a < 0.1) discard;
    
    // 📐 Нормализация нормали
    vec3 N = normalize(worldNormal);
    
    // 🔧 Материал
    float roughness, metallic, emission;
    getMaterial(albedo.rgb, roughness, metallic, emission);
    
    // 💡 Vanilla lightmap
    vec3 lightmapColor = texture(lightmap, lightmapCoord).rgb;
    
    // 🔮 Получение паттерна
    int patternIdx = getPatternIndex(worldPos);
    LightPattern pattern = fetchPattern(patternIdx);
    
    // ☀️ Направление на солнце
    vec3 sunDir = normalize(sunPosition);
    float dayFactor = max(0.0, sunDir.y);
    
    // 🔮 SH освещение
    vec3 shDiffuse = evaluateSHDiffuse(pattern.shCoeffs, N);
    
    // 🌑 Тени из SH
    float shShadow = computeSHShadow(pattern.shCoeffs, sunDir);
    
    // 💡 Прямое освещение
    vec3 directLight = pattern.directLight * shShadow * dayFactor;
    directLight *= (1.0 - rainStrength * 0.5);
    
    // 🌙 Непрямое освещение (GI)
    vec3 indirectLight = pattern.indirectLight * shDiffuse;
    
    // 🎨 Финальное освещение
    vec3 finalLight = directLight + indirectLight;
    finalLight *= pattern.ao;
    
    // 📦 Вывод в G-buffer
    outColor = vec4(albedo.rgb, 1.0);
    outNormal = vec4(N * 0.5 + 0.5, roughness);
    outLightmap = vec4(lightmapColor.rg, metallic, emission);
    
    // 🔮 SH lighting result (для composite pass)
    outShLight = vec4(finalLight, 1.0);
}
