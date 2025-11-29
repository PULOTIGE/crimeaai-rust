package net.voxelcrai.config;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;
import net.voxelcrai.mod.VoxelCraiMod;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * ⚙️ VoxelCraiConfig - Конфигурация мода
 * 
 * Настраиваемые параметры:
 * - patternCount: количество паттернов (1k-10k)
 * - shBands: количество SH bands (3-5)
 * - patternDensity: плотность паттернов (1-4)
 * - maxPatternsPerChunk: лимит на чанк
 * - enableReflections: включить отражения
 * - enableShadows: включить тени
 * - debugMode: режим отладки
 */
public class VoxelCraiConfig {
    
    // 📏 Границы значений
    public static final int MIN_PATTERN_COUNT = 1_000;
    public static final int MAX_PATTERN_COUNT = 100_000;
    public static final int MIN_SH_BANDS = 3;
    public static final int MAX_SH_BANDS = 5;
    
    // 🔧 Параметры паттернов
    private int patternCount = 10_000;
    private int shBands = 4;
    private int patternDensity = 2;  // 1 = каждый блок, 2 = каждый второй
    private int maxPatternsPerChunk = 512;
    
    // 🎨 Параметры рендеринга
    private boolean enableReflections = true;
    private boolean enableShadows = true;
    private boolean enableGI = true;
    private float giIntensity = 1.0f;
    private float shadowIntensity = 1.0f;
    private float reflectionIntensity = 0.8f;
    
    // 🔧 Производительность
    private boolean asyncPatternGeneration = true;
    private int updateIntervalTicks = 20;
    
    // 🐛 Отладка
    private boolean debugMode = false;
    private boolean showPatternCount = false;
    private boolean visualizeSH = false;
    
    // 📦 Сериализация
    private static final Gson GSON = new GsonBuilder()
        .setPrettyPrinting()
        .create();
    
    /**
     * 📂 Загрузка конфигурации из файла
     */
    public static VoxelCraiConfig load() {
        Path configPath = getConfigPath();
        
        if (Files.exists(configPath)) {
            try {
                String json = Files.readString(configPath);
                VoxelCraiConfig config = GSON.fromJson(json, VoxelCraiConfig.class);
                config.validate();
                VoxelCraiMod.LOGGER.info("⚙️ Конфигурация загружена: {}", configPath);
                return config;
            } catch (IOException e) {
                VoxelCraiMod.LOGGER.error("❌ Ошибка загрузки конфигурации: {}", e.getMessage());
            }
        }
        
        // Создаем конфигурацию по умолчанию
        VoxelCraiConfig config = new VoxelCraiConfig();
        config.save();
        return config;
    }
    
    /**
     * 💾 Сохранение конфигурации в файл
     */
    public void save() {
        Path configPath = getConfigPath();
        
        try {
            Files.createDirectories(configPath.getParent());
            Files.writeString(configPath, GSON.toJson(this));
            VoxelCraiMod.LOGGER.info("💾 Конфигурация сохранена: {}", configPath);
        } catch (IOException e) {
            VoxelCraiMod.LOGGER.error("❌ Ошибка сохранения конфигурации: {}", e.getMessage());
        }
    }
    
    /**
     * ✅ Валидация параметров
     */
    private void validate() {
        patternCount = clamp(patternCount, MIN_PATTERN_COUNT, MAX_PATTERN_COUNT);
        shBands = clamp(shBands, MIN_SH_BANDS, MAX_SH_BANDS);
        patternDensity = clamp(patternDensity, 1, 4);
        maxPatternsPerChunk = clamp(maxPatternsPerChunk, 64, 2048);
        giIntensity = clamp(giIntensity, 0.0f, 2.0f);
        shadowIntensity = clamp(shadowIntensity, 0.0f, 2.0f);
        reflectionIntensity = clamp(reflectionIntensity, 0.0f, 1.0f);
        updateIntervalTicks = clamp(updateIntervalTicks, 1, 100);
    }
    
    /**
     * 📂 Получение пути к файлу конфигурации
     */
    private static Path getConfigPath() {
        return FabricLoader.getInstance()
            .getConfigDir()
            .resolve(VoxelCraiMod.MOD_ID + ".json");
    }
    
    // ========== 🔧 Геттеры и сеттеры ==========
    
    public int getPatternCount() { return patternCount; }
    public void setPatternCount(int count) { 
        this.patternCount = clamp(count, MIN_PATTERN_COUNT, MAX_PATTERN_COUNT); 
    }
    
    public int getShBands() { return shBands; }
    public void setShBands(int bands) { 
        this.shBands = clamp(bands, MIN_SH_BANDS, MAX_SH_BANDS); 
    }
    
    public int getPatternDensity() { return patternDensity; }
    public void setPatternDensity(int density) { 
        this.patternDensity = clamp(density, 1, 4); 
    }
    
    public int getMaxPatternsPerChunk() { return maxPatternsPerChunk; }
    public void setMaxPatternsPerChunk(int max) { 
        this.maxPatternsPerChunk = clamp(max, 64, 2048); 
    }
    
    public boolean isEnableReflections() { return enableReflections; }
    public void setEnableReflections(boolean enable) { this.enableReflections = enable; }
    
    public boolean isEnableShadows() { return enableShadows; }
    public void setEnableShadows(boolean enable) { this.enableShadows = enable; }
    
    public boolean isEnableGI() { return enableGI; }
    public void setEnableGI(boolean enable) { this.enableGI = enable; }
    
    public float getGiIntensity() { return giIntensity; }
    public void setGiIntensity(float intensity) { 
        this.giIntensity = clamp(intensity, 0.0f, 2.0f); 
    }
    
    public float getShadowIntensity() { return shadowIntensity; }
    public void setShadowIntensity(float intensity) { 
        this.shadowIntensity = clamp(intensity, 0.0f, 2.0f); 
    }
    
    public float getReflectionIntensity() { return reflectionIntensity; }
    public void setReflectionIntensity(float intensity) { 
        this.reflectionIntensity = clamp(intensity, 0.0f, 1.0f); 
    }
    
    public boolean isAsyncPatternGeneration() { return asyncPatternGeneration; }
    public void setAsyncPatternGeneration(boolean async) { this.asyncPatternGeneration = async; }
    
    public int getUpdateIntervalTicks() { return updateIntervalTicks; }
    public void setUpdateIntervalTicks(int ticks) { 
        this.updateIntervalTicks = clamp(ticks, 1, 100); 
    }
    
    public boolean isDebugMode() { return debugMode; }
    public void setDebugMode(boolean debug) { this.debugMode = debug; }
    
    public boolean isShowPatternCount() { return showPatternCount; }
    public void setShowPatternCount(boolean show) { this.showPatternCount = show; }
    
    public boolean isVisualizeSH() { return visualizeSH; }
    public void setVisualizeSH(boolean visualize) { this.visualizeSH = visualize; }
    
    // ========== 🔧 Утилиты ==========
    
    private static int clamp(int value, int min, int max) {
        return Math.max(min, Math.min(max, value));
    }
    
    private static float clamp(float value, float min, float max) {
        return Math.max(min, Math.min(max, value));
    }
}
