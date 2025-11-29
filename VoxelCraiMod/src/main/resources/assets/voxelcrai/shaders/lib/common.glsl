// 🔧 VoxelCrai Common Library
// Общие функции и константы
// ТОЧКА ВХОДА для всех шейдеров

#ifndef VOXELCRAI_COMMON_GLSL
#define VOXELCRAI_COMMON_GLSL

// ============================================
// 📐 Математические константы
// ============================================

const float PI = 3.14159265358979323846;
const float TWO_PI = 6.28318530717958647692;
const float HALF_PI = 1.57079632679489661923;
const float INV_PI = 0.31830988618379067154;
const float INV_TWO_PI = 0.15915494309189533577;

const float EPSILON = 1e-6;
const float FLT_MAX = 3.402823466e+38;

// ============================================
// 🔢 Утилиты
// ============================================

// 📐 Saturate (clamp to [0, 1])
float saturate(float x) {
    return clamp(x, 0.0, 1.0);
}

vec2 saturate(vec2 x) {
    return clamp(x, vec2(0.0), vec2(1.0));
}

vec3 saturate(vec3 x) {
    return clamp(x, vec3(0.0), vec3(1.0));
}

// 📐 Square
float sq(float x) {
    return x * x;
}

// 📐 Luminance (sRGB)
float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

// 📐 Linear to sRGB
vec3 linearToSrgb(vec3 color) {
    return pow(color, vec3(1.0 / 2.2));
}

// 📐 sRGB to Linear
vec3 srgbToLinear(vec3 color) {
    return pow(color, vec3(2.2));
}

// ============================================
// 🎨 Цветовые пространства
// ============================================

// RGB -> XYZ (D65)
vec3 rgbToXyz(vec3 rgb) {
    mat3 m = mat3(
        0.4124564, 0.3575761, 0.1804375,
        0.2126729, 0.7151522, 0.0721750,
        0.0193339, 0.1191920, 0.9503041
    );
    return m * rgb;
}

// XYZ -> RGB (D65)
vec3 xyzToRgb(vec3 xyz) {
    mat3 m = mat3(
         3.2404542, -1.5371385, -0.4985314,
        -0.9692660,  1.8760108,  0.0415560,
         0.0556434, -0.2040259,  1.0572252
    );
    return m * xyz;
}

// ============================================
// 📦 Упаковка/распаковка
// ============================================

// 🔢 Упаковка нормали в oct encoding
vec2 octEncode(vec3 n) {
    n /= (abs(n.x) + abs(n.y) + abs(n.z));
    vec2 oct = n.xy;
    if (n.z < 0.0) {
        oct = (1.0 - abs(oct.yx)) * vec2(oct.x >= 0.0 ? 1.0 : -1.0, oct.y >= 0.0 ? 1.0 : -1.0);
    }
    return oct * 0.5 + 0.5;
}

// 🔢 Распаковка нормали из oct encoding
vec3 octDecode(vec2 oct) {
    oct = oct * 2.0 - 1.0;
    vec3 n = vec3(oct.xy, 1.0 - abs(oct.x) - abs(oct.y));
    if (n.z < 0.0) {
        n.xy = (1.0 - abs(n.yx)) * vec2(n.x >= 0.0 ? 1.0 : -1.0, n.y >= 0.0 ? 1.0 : -1.0);
    }
    return normalize(n);
}

// ============================================
// 🎲 Псевдослучайные числа
// ============================================

// 🎲 Hash функция
uint hash(uint x) {
    x ^= x >> 16;
    x *= 0x7feb352du;
    x ^= x >> 15;
    x *= 0x846ca68bu;
    x ^= x >> 16;
    return x;
}

// 🎲 Float из uint [0, 1)
float uintToFloat01(uint x) {
    return float(x) * (1.0 / 4294967296.0);
}

// 🎲 Random float [0, 1)
float random(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

// 🎲 3D noise hash
float hash31(vec3 p) {
    p = fract(p * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yxz + 33.33);
    return fract((p.x + p.y) * p.z);
}

// ============================================
// 📐 Координатные преобразования
// ============================================

// 📐 Depth -> Linear depth
float linearizeDepth(float depth, float near, float far) {
    return (2.0 * near) / (far + near - depth * (far - near));
}

// 📐 Clip -> View position
vec3 clipToView(vec3 clipPos, mat4 projInverse) {
    vec4 viewPos = projInverse * vec4(clipPos, 1.0);
    return viewPos.xyz / viewPos.w;
}

// 📐 Screen UV -> World position
vec3 screenToWorld(vec2 uv, float depth, mat4 projInverse, mat4 viewInverse, vec3 camPos) {
    vec3 clipPos = vec3(uv * 2.0 - 1.0, depth * 2.0 - 1.0);
    vec3 viewPos = clipToView(clipPos, projInverse);
    return (viewInverse * vec4(viewPos, 1.0)).xyz + camPos;
}

#endif // VOXELCRAI_COMMON_GLSL
