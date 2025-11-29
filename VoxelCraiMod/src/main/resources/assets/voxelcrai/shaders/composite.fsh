#version 430 core
// 🚀 VoxelCrai - Composite Fragment Shader
// Финальная сборка освещения из G-buffer и SH паттернов

in vec2 texCoord;

layout(location = 0) out vec4 outColor;

// 🖼️ G-buffer текстуры
uniform sampler2D colortex0;  // albedo
uniform sampler2D colortex1;  // normal + roughness
uniform sampler2D colortex2;  // lightmap + metallic + emission
uniform sampler2D colortex3;  // SH lighting
uniform sampler2D depthtex0;  // depth

// 🌍 Униформы
uniform mat4 gbufferProjectionInverse;
uniform mat4 gbufferModelViewInverse;
uniform vec3 cameraPosition;
uniform vec3 sunPosition;
uniform float rainStrength;
uniform int worldTime;

// 🔧 Конфигурация (из shaders.properties)
uniform int shBands;
uniform float giIntensity;
uniform float shadowIntensity;
uniform float reflectionIntensity;

// ============================================
// 🎨 Tonemapping
// ============================================

// ACES Filmic Tone Mapping
vec3 ACESFilm(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// Reinhard Extended
vec3 reinhardExtended(vec3 color, float maxWhite) {
    vec3 numerator = color * (1.0 + color / (maxWhite * maxWhite));
    return numerator / (1.0 + color);
}

// ============================================
// 💡 PBR функции
// ============================================

// Fresnel-Schlick
vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// ============================================
// 🎯 Main
// ============================================

void main() {
    // 📖 Чтение G-buffer
    vec4 albedoData = texture(colortex0, texCoord);
    vec4 normalData = texture(colortex1, texCoord);
    vec4 lightmapData = texture(colortex2, texCoord);
    vec4 shLighting = texture(colortex3, texCoord);
    float depth = texture(depthtex0, texCoord).r;
    
    // 🌌 Небо - без обработки
    if (depth >= 1.0) {
        // Простое небо
        vec3 skyTop = vec3(0.3, 0.5, 0.9);
        vec3 skyBottom = vec3(0.6, 0.7, 0.9);
        float skyGradient = texCoord.y;
        vec3 skyColor = mix(skyBottom, skyTop, skyGradient);
        
        // Время суток
        float dayTime = float(worldTime) / 24000.0;
        float sunFactor = max(0.0, sin(dayTime * 6.28318));
        skyColor *= 0.3 + sunFactor * 0.7;
        
        outColor = vec4(skyColor, 1.0);
        return;
    }
    
    // 📐 Данные из G-buffer
    vec3 albedo = albedoData.rgb;
    vec3 normal = normalData.rgb * 2.0 - 1.0;
    float roughness = normalData.a;
    float metallic = lightmapData.b;
    float emission = lightmapData.a;
    
    // 💡 SH освещение (уже вычислено в terrain pass)
    vec3 lighting = shLighting.rgb;
    
    // 🔧 Применение интенсивностей из конфига
    lighting *= giIntensity;
    
    // 🎨 Применение albedo
    // Metallic workflow
    vec3 F0 = mix(vec3(0.04), albedo, metallic);
    vec3 diffuse = albedo * (1.0 - metallic);
    
    // 💡 Финальный цвет
    vec3 finalColor = diffuse * lighting;
    
    // ✨ Specular (простая аппроксимация)
    vec3 viewDir = vec3(0.0, 0.0, 1.0);  // Упрощенно
    vec3 halfVec = normalize(viewDir + normalize(sunPosition));
    float NdotH = max(dot(normal, halfVec), 0.0);
    float spec = pow(NdotH, mix(8.0, 256.0, 1.0 - roughness));
    vec3 specular = F0 * spec * reflectionIntensity * (1.0 - roughness);
    
    finalColor += specular;
    
    // 🔥 Эмиссия
    finalColor += albedo * emission * 2.0;
    
    // 🌅 Ambient
    vec3 ambient = albedo * 0.02;
    finalColor += ambient;
    
    // 🎨 Tonemapping
    finalColor = ACESFilm(finalColor);
    
    // 🌈 Гамма-коррекция
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    outColor = vec4(finalColor, 1.0);
}
