package net.voxelcrai.shader;

import net.fabricmc.fabric.api.resource.SimpleSynchronousResourceReloadListener;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import net.voxelcrai.config.VoxelCraiConfig;
import net.voxelcrai.mod.VoxelCraiMod;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

/**
 * 🎭 ShaderPackManager - Менеджер шейдер-пака Iris
 * 
 * Функции:
 * - Генерация/обновление шейдер-пака при загрузке ресурсов
 * - Динамическое обновление SSBO буфера с паттернами
 * - Интеграция с Iris Shaders API
 * 
 * Структура шейдер-пака:
 * - shaders/shaders.properties
 * - shaders/program.json (Iris 1.7+)
 * - shaders/core/voxelcrai.vsh
 * - shaders/core/voxelcrai.fsh
 * - shaders/lib/sh_eval.glsl
 * - shaders/lib/ubos.glsl
 * - shaders/lib/patterns.glsl
 */
public class ShaderPackManager implements SimpleSynchronousResourceReloadListener {
    
    private static final String SHADER_PACK_NAME = "VoxelCrai-SH-Lighting";
    
    private final VoxelCraiConfig config;
    private Path shaderPackPath;
    private boolean initialized;
    
    /**
     * 🏗️ Конструктор
     */
    public ShaderPackManager(VoxelCraiConfig config) {
        this.config = config;
        this.initialized = false;
    }
    
    @Override
    public Identifier getFabricId() {
        return Identifier.of(VoxelCraiMod.MOD_ID, "shader_pack_manager");
    }
    
    @Override
    public void reload(ResourceManager manager) {
        VoxelCraiMod.LOGGER.info("🎭 Перезагрузка шейдер-пака...");
        
        try {
            generateShaderPack();
            initialized = true;
            VoxelCraiMod.LOGGER.info("✅ Шейдер-пак обновлен: {}", shaderPackPath);
        } catch (IOException e) {
            VoxelCraiMod.LOGGER.error("❌ Ошибка генерации шейдер-пака: {}", e.getMessage());
        }
    }
    
    /**
     * 📦 Генерация шейдер-пака
     */
    public void generateShaderPack() throws IOException {
        // Путь к папке shaderpacks в .minecraft
        Path minecraftDir = net.fabricmc.loader.api.FabricLoader.getInstance()
            .getGameDir();
        Path shaderpacksDir = minecraftDir.resolve("shaderpacks");
        Files.createDirectories(shaderpacksDir);
        
        shaderPackPath = shaderpacksDir.resolve(SHADER_PACK_NAME + ".zip");
        
        VoxelCraiMod.LOGGER.info("📦 Генерация шейдер-пака: {}", shaderPackPath);
        
        try (ZipOutputStream zos = new ZipOutputStream(
                new BufferedOutputStream(Files.newOutputStream(shaderPackPath)))) {
            
            // 📋 shaders.properties
            addTextEntry(zos, "shaders/shaders.properties", generateShadersProperties());
            
            // 📝 Основные шейдеры
            addTextEntry(zos, "shaders/gbuffers_terrain.vsh", generateTerrainVsh());
            addTextEntry(zos, "shaders/gbuffers_terrain.fsh", generateTerrainFsh());
            
            addTextEntry(zos, "shaders/composite.vsh", generateCompositeVsh());
            addTextEntry(zos, "shaders/composite.fsh", generateCompositeFsh());
            
            addTextEntry(zos, "shaders/final.vsh", generateFinalVsh());
            addTextEntry(zos, "shaders/final.fsh", generateFinalFsh());
            
            // 📚 Библиотеки
            addTextEntry(zos, "shaders/lib/sh_eval.glsl", generateShEvalLib());
            addTextEntry(zos, "shaders/lib/ubos.glsl", generateUbosLib());
            addTextEntry(zos, "shaders/lib/patterns.glsl", generatePatternsLib());
            addTextEntry(zos, "shaders/lib/lighting.glsl", generateLightingLib());
            addTextEntry(zos, "shaders/lib/materials.glsl", generateMaterialsLib());
            
            VoxelCraiMod.LOGGER.info("✅ Шейдер-пак создан успешно");
        }
    }
    
    /**
     * 📦 Добавление текстовой записи в ZIP
     */
    private void addTextEntry(ZipOutputStream zos, String path, String content) throws IOException {
        ZipEntry entry = new ZipEntry(path);
        zos.putNextEntry(entry);
        zos.write(content.getBytes("UTF-8"));
        zos.closeEntry();
    }
    
    // ========== 📋 Генераторы шейдеров ==========
    
    /**
     * 📋 shaders.properties
     */
    private String generateShadersProperties() {
        return String.format("""
            # 🚀 VoxelCrai SH Lighting Shader Pack
            # Воксельное освещение на основе Spherical Harmonics паттернов
            # Без трассировки лучей - чистые SH паттерны для 60+ FPS
            
            # ========== Базовые настройки ==========
            version=1.0.0
            profile.MEDIUM=
            profile.HIGH=shadowQuality:1 shadowMapResolution:2048
            profile.ULTRA=shadowQuality:2 shadowMapResolution:4096 shBands:4
            
            # ========== Тени ==========
            shadow.enabled=true
            shadowMapResolution=1024
            shadowDistance=128
            shadowQuality=0
            
            # ========== SH паттерны ==========
            # Количество SH bands (3-5)
            variable.int.shBands=%d
            sliderOptions=shBands
            shBands.comment=SH Bands: 3=быстро, 4=баланс, 5=качество
            
            # ========== GI настройки ==========
            variable.float.giIntensity=%.2f
            variable.float.shadowIntensity=%.2f
            variable.float.reflectionIntensity=%.2f
            
            # ========== Производительность ==========
            # Оптимизировано для AMD Radeon VII
            allowConcurrentCompute=true
            
            # ========== Буферы ==========
            # colortex0: albedo (RGB)
            # colortex1: normal (RGB) + roughness (A)
            # colortex2: lightmap (RG) + metallic (B) + emission (A)
            # colortex3: composite (RGBA16F)
            # colortex4: SH lighting (RGBA16F)
            
            <custom>
            [patterns]
            // 🔮 Количество паттернов: %d
            // 📊 Память SSBO: %d KB
            </custom>
            """,
            config.getShBands(),
            config.getGiIntensity(),
            config.getShadowIntensity(),
            config.getReflectionIntensity(),
            config.getPatternCount(),
            config.getPatternCount()  // 1KB per pattern
        );
    }
    
    /**
     * 🎨 gbuffers_terrain.vsh - Вершинный шейдер террейна
     */
    private String generateTerrainVsh() {
        return """
            #version 330 core
            // 🚀 VoxelCrai SH Lighting - Terrain Vertex Shader
            // Воксельное освещение на основе Spherical Harmonics
            
            #include "lib/ubos.glsl"
            
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
            
            // 🌍 Униформы
            uniform mat4 modelViewMatrix;
            uniform mat4 projectionMatrix;
            uniform mat4 gbufferModelViewInverse;
            uniform vec3 cameraPosition;
            
            void main() {
                // 📍 Позиция в мире
                vec4 viewPos = modelViewMatrix * vec4(vaPosition, 1.0);
                worldPos = (gbufferModelViewInverse * viewPos).xyz + cameraPosition;
                
                // 📐 Нормаль в мировых координатах
                worldNormal = normalize(mat3(gbufferModelViewInverse) * vaNormal);
                
                // 📝 Передача данных
                texCoord = vaUV0;
                vertexColor = vaColor;
                lightmapCoord = vaUV2 / 256.0;  // Нормализация lightmap
                
                // 🎯 Финальная позиция
                gl_Position = projectionMatrix * viewPos;
                depth = gl_Position.z / gl_Position.w;
            }
            """;
    }
    
    /**
     * 🎨 gbuffers_terrain.fsh - Фрагментный шейдер террейна
     */
    private String generateTerrainFsh() {
        return """
            #version 330 core
            // 🚀 VoxelCrai SH Lighting - Terrain Fragment Shader
            // Запись в G-buffer для отложенного освещения
            
            #include "lib/ubos.glsl"
            #include "lib/materials.glsl"
            
            // 📥 Входные данные из вершинного шейдера
            in vec2 texCoord;
            in vec3 worldPos;
            in vec3 worldNormal;
            in vec4 vertexColor;
            in vec2 lightmapCoord;
            in float depth;
            
            // 📤 Выходные буферы (G-buffer)
            layout(location = 0) out vec4 outColor;      // colortex0: albedo
            layout(location = 1) out vec4 outNormal;     // colortex1: normal + roughness
            layout(location = 2) out vec4 outLightmap;   // colortex2: lightmap + metallic + emission
            
            // 🖼️ Текстуры
            uniform sampler2D gtexture;
            uniform sampler2D lightmap;
            
            void main() {
                // 🎨 Базовый цвет
                vec4 albedo = texture(gtexture, texCoord) * vertexColor;
                
                // ✂️ Alpha test
                if (albedo.a < 0.1) discard;
                
                // 🔧 Материал (на основе цвета/позиции)
                MaterialProps mat = getMaterialFromColor(albedo.rgb);
                
                // 💡 Lightmap
                vec2 lmCoord = lightmapCoord;
                vec3 lightmapColor = texture(lightmap, lmCoord).rgb;
                
                // 📦 Запись в G-buffer
                outColor = vec4(albedo.rgb, 1.0);
                outNormal = vec4(worldNormal * 0.5 + 0.5, mat.roughness);
                outLightmap = vec4(lightmapColor.rg, mat.metallic, mat.emission);
            }
            """;
    }
    
    /**
     * 🌟 composite.vsh - Вершинный шейдер для пост-обработки
     */
    private String generateCompositeVsh() {
        return """
            #version 330 core
            // 🚀 VoxelCrai SH Lighting - Composite Vertex Shader
            // Fullscreen quad для SH освещения
            
            out vec2 texCoord;
            
            void main() {
                // 📐 Fullscreen quad (2 треугольника)
                const vec2 positions[4] = vec2[4](
                    vec2(-1.0, -1.0),
                    vec2( 1.0, -1.0),
                    vec2(-1.0,  1.0),
                    vec2( 1.0,  1.0)
                );
                
                vec2 pos = positions[gl_VertexID];
                texCoord = pos * 0.5 + 0.5;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
            """;
    }
    
    /**
     * 🌟 composite.fsh - Главный фрагментный шейдер с SH освещением
     */
    private String generateCompositeFsh() {
        return String.format("""
            #version 430 core
            // 🚀 VoxelCrai SH Lighting - Composite Fragment Shader
            // Основной шейдер с SH-based GI, тенями и отражениями
            
            #include "lib/ubos.glsl"
            #include "lib/sh_eval.glsl"
            #include "lib/patterns.glsl"
            #include "lib/lighting.glsl"
            
            // 📥 Входные данные
            in vec2 texCoord;
            
            // 📤 Выходной буфер
            layout(location = 0) out vec4 outColor;
            
            // 🖼️ G-buffer текстуры
            uniform sampler2D colortex0;  // albedo
            uniform sampler2D colortex1;  // normal + roughness
            uniform sampler2D colortex2;  // lightmap + metallic + emission
            uniform sampler2D depthtex0;  // depth
            
            // 🌍 Униформы
            uniform mat4 gbufferProjectionInverse;
            uniform mat4 gbufferModelViewInverse;
            uniform vec3 cameraPosition;
            uniform vec3 sunPosition;
            uniform float rainStrength;
            uniform int worldTime;
            
            // 🔧 Конфигурация
            const int SH_BANDS = %d;
            const float GI_INTENSITY = %.2f;
            const float SHADOW_INTENSITY = %.2f;
            const float REFLECTION_INTENSITY = %.2f;
            
            void main() {
                // 📖 Чтение G-buffer
                vec4 albedoData = texture(colortex0, texCoord);
                vec4 normalData = texture(colortex1, texCoord);
                vec4 lightmapData = texture(colortex2, texCoord);
                float depth = texture(depthtex0, texCoord).r;
                
                // 🌌 Если небо - просто возвращаем цвет
                if (depth >= 1.0) {
                    outColor = albedoData;
                    return;
                }
                
                // 📐 Реконструкция нормали
                vec3 normal = normalData.rgb * 2.0 - 1.0;
                float roughness = normalData.a;
                float metallic = lightmapData.b;
                float emission = lightmapData.a;
                
                // 📍 Реконструкция мировой позиции
                vec4 clipPos = vec4(texCoord * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
                vec4 viewPos = gbufferProjectionInverse * clipPos;
                viewPos /= viewPos.w;
                vec3 worldPos = (gbufferModelViewInverse * viewPos).xyz + cameraPosition;
                
                // 🔮 Получение паттерна по позиции
                int patternIdx = getPatternIndex(worldPos);
                LightPattern pattern = fetchPattern(patternIdx);
                
                // ☀️ Направление на солнце
                vec3 sunDir = normalize(sunPosition);
                float dayFactor = max(0.0, sunDir.y);
                
                // 🔮 SH освещение
                vec3 shDiffuse = evaluateSH(pattern.shCoeffs, normal, SH_BANDS);
                
                // 🌑 Тени из SH (негативные коэффициенты = окклюзия)
                float shShadow = computeSHShadow(pattern.shCoeffs, sunDir);
                shShadow = mix(1.0, shShadow, SHADOW_INTENSITY);
                
                // 💡 Прямое освещение
                vec3 directLight = pattern.directLight * shShadow * dayFactor;
                directLight *= (1.0 - rainStrength * 0.5);  // Дождь ослабляет
                
                // 🌙 Непрямое освещение (GI)
                vec3 indirectLight = pattern.indirectLight * shDiffuse * GI_INTENSITY;
                
                // ✨ Specular отражения (аппроксимация через SH)
                vec3 viewDir = normalize(cameraPosition - worldPos);
                vec3 reflectDir = reflect(-viewDir, normal);
                vec3 specular = evaluateSHSpecular(
                    pattern.shCoeffs, 
                    reflectDir, 
                    roughness, 
                    metallic,
                    SH_BANDS
                ) * REFLECTION_INTENSITY;
                
                // 🎨 Финальное освещение
                vec3 albedo = albedoData.rgb;
                
                // Metallic workflow
                vec3 F0 = mix(vec3(0.04), albedo, metallic);
                vec3 diffuse = albedo * (1.0 - metallic);
                
                // 💡 Комбинация
                vec3 lighting = diffuse * (directLight + indirectLight);
                lighting += specular * F0;
                
                // 🔥 Эмиссия
                lighting += albedo * emission * 2.0;
                
                // 🌅 Ambient
                float ao = pattern.ambientOcclusion;
                vec3 ambient = albedo * 0.03 * ao;
                lighting += ambient;
                
                outColor = vec4(lighting, 1.0);
            }
            """,
            config.getShBands(),
            config.getGiIntensity(),
            config.getShadowIntensity(),
            config.getReflectionIntensity()
        );
    }
    
    /**
     * 🎬 final.vsh - Финальный вершинный шейдер
     */
    private String generateFinalVsh() {
        return """
            #version 330 core
            // 🚀 VoxelCrai - Final Pass Vertex Shader
            
            out vec2 texCoord;
            
            void main() {
                const vec2 positions[4] = vec2[4](
                    vec2(-1.0, -1.0),
                    vec2( 1.0, -1.0),
                    vec2(-1.0,  1.0),
                    vec2( 1.0,  1.0)
                );
                
                vec2 pos = positions[gl_VertexID];
                texCoord = pos * 0.5 + 0.5;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
            """;
    }
    
    /**
     * 🎬 final.fsh - Финальный фрагментный шейдер (тонмаппинг)
     */
    private String generateFinalFsh() {
        return """
            #version 330 core
            // 🚀 VoxelCrai - Final Pass Fragment Shader
            // Тонмаппинг и гамма-коррекция
            
            in vec2 texCoord;
            
            layout(location = 0) out vec4 outColor;
            
            uniform sampler2D colortex3;  // HDR result from composite
            
            // 🎨 ACES Tonemapping
            vec3 ACESFilm(vec3 x) {
                float a = 2.51;
                float b = 0.03;
                float c = 2.43;
                float d = 0.59;
                float e = 0.14;
                return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
            }
            
            void main() {
                vec3 hdrColor = texture(colortex3, texCoord).rgb;
                
                // 🎨 Tonemapping
                vec3 mapped = ACESFilm(hdrColor);
                
                // 🌈 Гамма-коррекция
                mapped = pow(mapped, vec3(1.0 / 2.2));
                
                outColor = vec4(mapped, 1.0);
            }
            """;
    }
    
    /**
     * 🔮 lib/sh_eval.glsl - Библиотека SH функций
     */
    private String generateShEvalLib() {
        return """
            // 🔮 SH Evaluation Library
            // Spherical Harmonics базисные функции и реконструкция
            // Портировано из Rust прототипа
            
            #ifndef SH_EVAL_GLSL
            #define SH_EVAL_GLSL
            
            // 📐 SH константы (нормализующие множители)
            const float SH_C0 = 0.282095;      // Y00
            const float SH_C1 = 0.488603;      // Y1x
            const float SH_C2_0 = 1.092548;    // Y2-2, Y2-1, Y21
            const float SH_C2_1 = 0.315392;    // Y20
            const float SH_C2_2 = 0.546274;    // Y22
            const float SH_C3_0 = 0.590044;    // Y3-3, Y33
            const float SH_C3_1 = 2.890611;    // Y3-2, Y32
            const float SH_C3_2 = 0.457046;    // Y3-1, Y31
            const float SH_C3_3 = 0.373176;    // Y30
            
            /**
             * 🔮 Вычисление SH базисных функций для направления
             * @param dir нормализованное направление
             * @param bands количество bands (3-5)
             * @return массив значений базисных функций
             */
            float[16] computeSHBasis(vec3 dir, int bands) {
                float[16] basis;
                
                float x = dir.x;
                float y = dir.y;
                float z = dir.z;
                
                // 🎯 Band 0 (1 коэффициент)
                basis[0] = SH_C0;
                
                // 🎯 Band 1 (3 коэффициента)
                if (bands >= 1) {
                    basis[1] = SH_C1 * y;
                    basis[2] = SH_C1 * z;
                    basis[3] = SH_C1 * x;
                }
                
                // 🎯 Band 2 (5 коэффициентов)
                if (bands >= 2) {
                    basis[4] = SH_C2_0 * x * y;
                    basis[5] = SH_C2_0 * y * z;
                    basis[6] = SH_C2_1 * (3.0 * z * z - 1.0);
                    basis[7] = SH_C2_0 * x * z;
                    basis[8] = SH_C2_2 * (x * x - y * y);
                }
                
                // 🎯 Band 3 (7 коэффициентов)
                if (bands >= 3) {
                    basis[9]  = SH_C3_0 * y * (3.0 * x * x - y * y);
                    basis[10] = SH_C3_1 * x * y * z;
                    basis[11] = SH_C3_2 * y * (4.0 * z * z - x * x - y * y);
                    basis[12] = SH_C3_3 * z * (2.0 * z * z - 3.0 * x * x - 3.0 * y * y);
                    basis[13] = SH_C3_2 * x * (4.0 * z * z - x * x - y * y);
                    basis[14] = SH_C3_1 * z * (x * x - y * y);
                    basis[15] = SH_C3_0 * x * (x * x - 3.0 * y * y);
                }
                
                return basis;
            }
            
            /**
             * 🔮 Реконструкция значения из SH коэффициентов
             * @param coeffs SH коэффициенты (i8 -> float, нормализованы)
             * @param dir направление
             * @param bands количество bands
             * @return реконструированное значение (скаляр)
             */
            float reconstructSH(float[16] coeffs, vec3 dir, int bands) {
                float[16] basis = computeSHBasis(dir, bands);
                
                float result = 0.0;
                int numCoeffs = bands * bands;  // 1, 4, 9, 16 для bands 1, 2, 3, 4
                
                for (int i = 0; i < numCoeffs && i < 16; i++) {
                    result += coeffs[i] * basis[i];
                }
                
                return result;
            }
            
            /**
             * 🔮 Вычисление diffuse освещения из SH
             * Использует косинусную свертку (convolution с lambert BRDF)
             * 
             * @param coeffs SH коэффициенты паттерна
             * @param normal нормаль поверхности
             * @param bands количество bands
             * @return RGB diffuse освещение
             */
            vec3 evaluateSH(float[16] coeffs, vec3 normal, int bands) {
                // 🌐 Косинусная свертка для Lambertian diffuse
                // Множители для bands: [π, 2π/3, π/4, ...]
                float[4] cosineLobeCoeffs = float[4](
                    3.14159265,      // Band 0
                    2.09439510,      // Band 1  
                    0.78539816,      // Band 2
                    0.0              // Band 3 (малый вклад)
                );
                
                float[16] basis = computeSHBasis(normal, bands);
                
                float irradiance = 0.0;
                
                // Band 0
                irradiance += coeffs[0] * basis[0] * cosineLobeCoeffs[0];
                
                // Band 1
                if (bands >= 2) {
                    irradiance += (coeffs[1] * basis[1] + 
                                   coeffs[2] * basis[2] + 
                                   coeffs[3] * basis[3]) * cosineLobeCoeffs[1];
                }
                
                // Band 2
                if (bands >= 3) {
                    irradiance += (coeffs[4] * basis[4] + 
                                   coeffs[5] * basis[5] + 
                                   coeffs[6] * basis[6] +
                                   coeffs[7] * basis[7] +
                                   coeffs[8] * basis[8]) * cosineLobeCoeffs[2];
                }
                
                // Нормализация и клампинг
                irradiance = max(0.0, irradiance);
                
                // RGB (пока grayscale, можно расширить для цветного GI)
                return vec3(irradiance);
            }
            
            /**
             * 🌑 Вычисление тени из SH коэффициентов
             * Негативные коэффициенты означают окклюзию
             * 
             * @param coeffs SH коэффициенты
             * @param lightDir направление на источник света
             * @return множитель тени [0, 1]
             */
            float computeSHShadow(float[16] coeffs, vec3 lightDir) {
                float[16] basis = computeSHBasis(lightDir, 3);
                
                float visibility = 0.0;
                for (int i = 0; i < 9; i++) {
                    visibility += coeffs[i] * basis[i];
                }
                
                // Конвертация в [0, 1]
                return clamp(visibility * 0.5 + 0.5, 0.0, 1.0);
            }
            
            /**
             * ✨ Вычисление specular отражений из SH
             * Аппроксимация отражений через направленный SH запрос
             * 
             * @param coeffs SH коэффициенты
             * @param reflectDir направление отражения
             * @param roughness шероховатость поверхности
             * @param metallic металличность
             * @param bands количество bands
             * @return RGB specular
             */
            vec3 evaluateSHSpecular(float[16] coeffs, vec3 reflectDir, float roughness, float metallic, int bands) {
                // 🔮 Выбираем количество bands на основе roughness
                // Гладкие поверхности используют больше bands (детали)
                // Шероховатые - меньше (размытие)
                int effectiveBands = int(mix(float(bands), 1.0, roughness));
                
                float specValue = reconstructSH(coeffs, reflectDir, effectiveBands);
                
                // 📈 Коррекция на roughness (размытие)
                float lod = roughness * roughness * float(bands);
                specValue *= exp(-lod);
                
                // 🎨 Цвет specular (для металлов - цвет поверхности)
                vec3 specColor = mix(vec3(1.0), vec3(specValue), metallic);
                
                return specColor * max(0.0, specValue);
            }
            
            #endif // SH_EVAL_GLSL
            """;
    }
    
    /**
     * 📦 lib/ubos.glsl - Uniform Buffer Objects
     */
    private String generateUbosLib() {
        return """
            // 📦 Uniform Buffer Objects
            // Общие структуры и буферы для шейдеров
            
            #ifndef UBOS_GLSL
            #define UBOS_GLSL
            
            // 🌍 Информация о мире
            struct WorldInfo {
                vec3 sunPosition;
                float dayTime;
                vec3 moonPosition;
                float rainStrength;
                vec3 cameraPosition;
                float thunderStrength;
            };
            
            // 📷 Информация о камере
            struct CameraInfo {
                mat4 projection;
                mat4 projectionInverse;
                mat4 modelView;
                mat4 modelViewInverse;
                vec3 position;
                float near;
                float far;
            };
            
            #endif // UBOS_GLSL
            """;
    }
    
    /**
     * 🔮 lib/patterns.glsl - Работа с паттернами
     */
    private String generatePatternsLib() {
        return String.format("""
            // 🔮 Light Patterns Library
            // Структуры и функции для работы с LightPattern1KB
            
            #ifndef PATTERNS_GLSL
            #define PATTERNS_GLSL
            
            // 📏 Константы
            const int PATTERN_SIZE = 1024;  // байт
            const int MAX_PATTERNS = %d;
            const int SH_COEFFS_COUNT = 16;
            
            // 💡 Структура паттерна (адаптация LightPattern1KB из Rust)
            struct LightPattern {
                uint id;
                vec3 directLight;      // RGB fp16
                vec3 indirectLight;    // RGB fp16
                float[16] shCoeffs;    // SH коэффициенты
                float roughness;
                float metallic;
                float ambientOcclusion;
                float reflection;
                float refraction;
                float emission;
                ivec3 position;
            };
            
            // 📦 SSBO с паттернами (заполняется из Java)
            layout(std430, binding = 0) buffer PatternBuffer {
                uint patternData[];
            };
            
            // 🔍 Количество активных паттернов
            uniform int activePatternCount;
            
            /**
             * 📖 Чтение паттерна из SSBO
             * @param idx индекс паттерна
             * @return структура LightPattern
             */
            LightPattern fetchPattern(int idx) {
                LightPattern pattern;
                
                if (idx < 0 || idx >= activePatternCount) {
                    // Возвращаем дефолтный паттерн
                    pattern.id = 0u;
                    pattern.directLight = vec3(0.5);
                    pattern.indirectLight = vec3(0.2);
                    for (int i = 0; i < 16; i++) {
                        pattern.shCoeffs[i] = 0.0;
                    }
                    pattern.shCoeffs[0] = 0.5;  // DC компонента
                    pattern.roughness = 0.5;
                    pattern.metallic = 0.0;
                    pattern.ambientOcclusion = 1.0;
                    pattern.reflection = 0.0;
                    pattern.refraction = 0.0;
                    pattern.emission = 0.0;
                    pattern.position = ivec3(0);
                    return pattern;
                }
                
                // 📍 Смещение в буфере (1024 байта на паттерн = 256 uint)
                int offset = idx * 256;
                
                // 🆔 ID (8 байт = 2 uint)
                pattern.id = patternData[offset];
                
                // 📍 Пропуск padding (2 uint)
                offset += 4;
                
                // 💡 Direct light RGB (6 байт = 2 uint с упаковкой fp16)
                uint packed0 = patternData[offset];
                uint packed1 = patternData[offset + 1];
                pattern.directLight.r = unpackHalf2x16(packed0).x;
                pattern.directLight.g = unpackHalf2x16(packed0).y;
                pattern.directLight.b = unpackHalf2x16(packed1).x;
                offset += 2;
                
                // 🌙 Indirect light RGB
                packed0 = patternData[offset];
                packed1 = patternData[offset + 1];
                pattern.indirectLight.r = unpackHalf2x16(packed0).x;
                pattern.indirectLight.g = unpackHalf2x16(packed0).y;
                pattern.indirectLight.b = unpackHalf2x16(packed1).x;
                offset += 2;
                
                // 🔮 SH коэффициенты (256 байт = 64 uint)
                // i8 упакованы по 4 в uint
                for (int i = 0; i < 16; i += 4) {
                    uint packedSH = patternData[offset + i/4];
                    pattern.shCoeffs[i+0] = float(int(packedSH & 0xFFu) - 128) / 127.0;
                    pattern.shCoeffs[i+1] = float(int((packedSH >> 8) & 0xFFu) - 128) / 127.0;
                    pattern.shCoeffs[i+2] = float(int((packedSH >> 16) & 0xFFu) - 128) / 127.0;
                    pattern.shCoeffs[i+3] = float(int((packedSH >> 24) & 0xFFu) - 128) / 127.0;
                }
                offset += 64;
                
                // 🎨 Material data (пропускаем для упрощения)
                offset += 128;
                
                // 🔧 Roughness/Metallic
                uint packedMat = patternData[offset];
                pattern.roughness = unpackHalf2x16(packedMat).x;
                pattern.metallic = unpackHalf2x16(packedMat).y;
                offset += 1;
                
                // ✨ AO/Reflection/Refraction/Emission
                packed0 = patternData[offset];
                packed1 = patternData[offset + 1];
                pattern.ambientOcclusion = unpackHalf2x16(packed0).x;
                pattern.reflection = unpackHalf2x16(packed0).y;
                pattern.refraction = unpackHalf2x16(packed1).x;
                pattern.emission = unpackHalf2x16(packed1).y;
                
                return pattern;
            }
            
            /**
             * 🔍 Получение индекса паттерна по мировой позиции
             * Использует пространственное хеширование
             * 
             * @param worldPos мировая позиция
             * @return индекс паттерна в буфере
             */
            int getPatternIndex(vec3 worldPos) {
                // 📐 Пространственное хеширование
                ivec3 cell = ivec3(floor(worldPos / 4.0));  // Ячейки 4x4x4
                
                // 🔑 Хеш-функция
                int hash = cell.x * 73856093 ^ cell.y * 19349663 ^ cell.z * 83492791;
                hash = abs(hash);
                
                return hash %% activePatternCount;
            }
            
            /**
             * 🔍 Получение паттерна для UV координат (для fullscreen эффектов)
             * @param uv текстурные координаты [0, 1]
             * @return индекс паттерна
             */
            int getPatternIndexFromUV(vec2 uv, float depth) {
                // Простое распределение по экрану
                int x = int(uv.x * 64.0);
                int y = int(uv.y * 64.0);
                int z = int(depth * 16.0);
                
                int idx = (z * 64 * 64 + y * 64 + x) %% activePatternCount;
                return idx;
            }
            
            #endif // PATTERNS_GLSL
            """,
            config.getPatternCount()
        );
    }
    
    /**
     * 💡 lib/lighting.glsl - Функции освещения
     */
    private String generateLightingLib() {
        return """
            // 💡 Lighting Library
            // PBR освещение и вспомогательные функции
            
            #ifndef LIGHTING_GLSL
            #define LIGHTING_GLSL
            
            const float PI = 3.14159265359;
            
            /**
             * 🔆 Fresnel-Schlick аппроксимация
             */
            vec3 fresnelSchlick(float cosTheta, vec3 F0) {
                return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
            }
            
            /**
             * 📐 Distribution GGX
             */
            float distributionGGX(vec3 N, vec3 H, float roughness) {
                float a = roughness * roughness;
                float a2 = a * a;
                float NdotH = max(dot(N, H), 0.0);
                float NdotH2 = NdotH * NdotH;
                
                float nom = a2;
                float denom = (NdotH2 * (a2 - 1.0) + 1.0);
                denom = PI * denom * denom;
                
                return nom / denom;
            }
            
            /**
             * 🔧 Geometry Schlick-GGX
             */
            float geometrySchlickGGX(float NdotV, float roughness) {
                float r = roughness + 1.0;
                float k = (r * r) / 8.0;
                
                float nom = NdotV;
                float denom = NdotV * (1.0 - k) + k;
                
                return nom / denom;
            }
            
            /**
             * 🔧 Geometry Smith
             */
            float geometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
                float NdotV = max(dot(N, V), 0.0);
                float NdotL = max(dot(N, L), 0.0);
                float ggx2 = geometrySchlickGGX(NdotV, roughness);
                float ggx1 = geometrySchlickGGX(NdotL, roughness);
                
                return ggx1 * ggx2;
            }
            
            /**
             * 💡 Cook-Torrance BRDF
             */
            vec3 cookTorranceBRDF(
                vec3 albedo,
                vec3 N,
                vec3 V,
                vec3 L,
                float roughness,
                float metallic,
                vec3 lightColor
            ) {
                vec3 H = normalize(V + L);
                
                vec3 F0 = mix(vec3(0.04), albedo, metallic);
                
                float NDF = distributionGGX(N, H, roughness);
                float G = geometrySmith(N, V, L, roughness);
                vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);
                
                vec3 numerator = NDF * G * F;
                float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
                vec3 specular = numerator / denominator;
                
                vec3 kS = F;
                vec3 kD = vec3(1.0) - kS;
                kD *= 1.0 - metallic;
                
                float NdotL = max(dot(N, L), 0.0);
                
                return (kD * albedo / PI + specular) * lightColor * NdotL;
            }
            
            #endif // LIGHTING_GLSL
            """;
    }
    
    /**
     * 🎨 lib/materials.glsl - Материалы
     */
    private String generateMaterialsLib() {
        return """
            // 🎨 Materials Library
            // Определение материалов на основе цвета/блока
            
            #ifndef MATERIALS_GLSL
            #define MATERIALS_GLSL
            
            struct MaterialProps {
                float roughness;
                float metallic;
                float emission;
            };
            
            /**
             * 🎨 Определение материала по цвету
             */
            MaterialProps getMaterialFromColor(vec3 color) {
                MaterialProps mat;
                mat.roughness = 0.7;
                mat.metallic = 0.0;
                mat.emission = 0.0;
                
                // 🪨 Серые тона (камень)
                float gray = (color.r + color.g + color.b) / 3.0;
                float saturation = max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
                
                if (saturation < 0.1 && gray > 0.3 && gray < 0.7) {
                    mat.roughness = 0.9;
                    mat.metallic = 0.0;
                }
                
                // ⛏️ Золотистые/медные тона (металлы)
                if (color.r > 0.6 && color.g > 0.4 && color.g < 0.8 && color.b < 0.4) {
                    mat.roughness = 0.3;
                    mat.metallic = 0.9;
                }
                
                // 🔵 Синие тона с высокой яркостью (алмазы, лазурит)
                if (color.b > 0.5 && color.r < 0.4) {
                    mat.roughness = 0.2;
                    mat.metallic = 0.5;
                }
                
                // 🌿 Зеленые тона (растения)
                if (color.g > color.r && color.g > color.b) {
                    mat.roughness = 0.95;
                    mat.metallic = 0.0;
                }
                
                // 💧 Водные тона
                if (color.b > 0.6 && color.g > 0.5 && color.r < 0.3) {
                    mat.roughness = 0.05;
                    mat.metallic = 0.0;
                }
                
                // 🔥 Яркие тона (лава, огонь)
                if (color.r > 0.9 && color.g > 0.3 && color.g < 0.7) {
                    mat.emission = 1.0;
                }
                
                // 🌟 Светящиеся блоки
                float brightness = (color.r + color.g + color.b) / 3.0;
                if (brightness > 0.9) {
                    mat.emission = 0.5;
                }
                
                return mat;
            }
            
            #endif // MATERIALS_GLSL
            """;
    }
    
    // ========== 🔧 Геттеры ==========
    
    public Path getShaderPackPath() {
        return shaderPackPath;
    }
    
    public boolean isInitialized() {
        return initialized;
    }
}
