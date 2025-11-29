package net.voxelcrai.mod;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientChunkEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.resource.ResourceManagerHelper;
import net.minecraft.resource.ResourceType;
import net.voxelcrai.config.VoxelCraiConfig;
import net.voxelcrai.pattern.LightPatternBuffer;
import net.voxelcrai.pattern.PatternGenerator;
import net.voxelcrai.shader.ShaderPackManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * 🚀 ТОЧКА ВХОДА - VoxelCraiMod
 * Воксельное освещение на основе Spherical Harmonics паттернов
 * 
 * Архитектура:
 * - LightPattern1KB: 1024 байта на паттерн (SH коэффициенты + материалы)
 * - PatternGenerator: CPU генерация 1k-10k паттернов при загрузке мира/чанков
 * - ShaderPackManager: GLSL шейдеры с SH eval для GI/теней/отражений
 * - SSBO буфер: 10k паттернов = 10MB GPU памяти
 * 
 * Без трассировки лучей - чистые SH паттерны для 60+ FPS на Radeon VII
 */
public class VoxelCraiMod implements ClientModInitializer {
    
    // 🔧 Константы мода
    public static final String MOD_ID = "voxelcrai";
    public static final String MOD_NAME = "VoxelCraiMod";
    public static final String VERSION = "1.0.0";
    
    // 📊 Логгер
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_NAME);
    
    // 🎯 Синглтон инстансы
    private static VoxelCraiMod INSTANCE;
    private VoxelCraiConfig config;
    private LightPatternBuffer patternBuffer;
    private PatternGenerator patternGenerator;
    private ShaderPackManager shaderPackManager;
    
    // 🔄 Состояние мода
    private boolean initialized = false;
    private int tickCounter = 0;
    
    @Override
    public void onInitializeClient() {
        INSTANCE = this;
        
        LOGGER.info("╔════════════════════════════════════════════════════════════╗");
        LOGGER.info("║  🚀 VoxelCraiMod v{} - Инициализация...              ║", VERSION);
        LOGGER.info("║  SH-based GI • Тени • Отражения • Без RT              ║");
        LOGGER.info("╚════════════════════════════════════════════════════════════╝");
        
        // 🔧 Загрузка конфигурации
        config = VoxelCraiConfig.load();
        LOGGER.info("📋 Конфиг загружен: {} паттернов, {} SH bands", 
            config.getPatternCount(), config.getShBands());
        
        // 🎨 Инициализация буфера паттернов
        patternBuffer = new LightPatternBuffer(config.getPatternCount());
        LOGGER.info("💾 Буфер паттернов: {} KB", patternBuffer.getSizeKB());
        
        // 🔮 Инициализация генератора паттернов
        patternGenerator = new PatternGenerator(config);
        LOGGER.info("🔮 Генератор паттернов готов");
        
        // 🎭 Инициализация менеджера шейдеров
        shaderPackManager = new ShaderPackManager(config);
        LOGGER.info("🎭 Менеджер шейдеров готов");
        
        // 📦 Регистрация обработчиков событий
        registerEventHandlers();
        
        // 🔌 Регистрация ресурсов
        registerResources();
        
        initialized = true;
        LOGGER.info("✅ VoxelCraiMod инициализирован успешно!");
    }
    
    /**
     * 📦 Регистрация обработчиков событий Fabric
     */
    private void registerEventHandlers() {
        // 🌍 Загрузка чанка - генерация паттернов
        ClientChunkEvents.CHUNK_LOAD.register((world, chunk) -> {
            if (!initialized) return;
            
            // 🔮 Генерируем паттерны для нового чанка
            int chunkX = chunk.getPos().x;
            int chunkZ = chunk.getPos().z;
            
            LOGGER.debug("🌍 Чанк загружен: [{}, {}]", chunkX, chunkZ);
            
            // Асинхронная генерация паттернов для чанка
            patternGenerator.generateForChunkAsync(chunk, patterns -> {
                patternBuffer.updatePatterns(patterns);
                LOGGER.debug("✨ Паттерны обновлены для чанка [{}, {}]: {} шт", 
                    chunkX, chunkZ, patterns.size());
            });
        });
        
        // 🗑️ Выгрузка чанка - освобождение паттернов
        ClientChunkEvents.CHUNK_UNLOAD.register((world, chunk) -> {
            if (!initialized) return;
            
            int chunkX = chunk.getPos().x;
            int chunkZ = chunk.getPos().z;
            
            LOGGER.debug("🗑️ Чанк выгружен: [{}, {}]", chunkX, chunkZ);
            patternGenerator.releaseChunk(chunkX, chunkZ);
        });
        
        // ⏱️ Тик клиента - обновление паттернов
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            if (!initialized || client.world == null) return;
            
            tickCounter++;
            
            // Обновление каждые 20 тиков (1 секунда)
            if (tickCounter >= 20) {
                tickCounter = 0;
                
                // 🔄 Обновление динамических паттернов (время суток, погода)
                float timeOfDay = client.world.getTimeOfDay() / 24000.0f;
                float rainGradient = client.world.getRainGradient(1.0f);
                
                patternGenerator.updateDynamicPatterns(timeOfDay, rainGradient);
            }
        });
    }
    
    /**
     * 🔌 Регистрация ресурсов мода
     */
    private void registerResources() {
        ResourceManagerHelper.get(ResourceType.CLIENT_RESOURCES)
            .registerReloadListener(shaderPackManager);
        
        LOGGER.info("🔌 Ресурсы зарегистрированы");
    }
    
    // ========== Геттеры ==========
    
    public static VoxelCraiMod getInstance() {
        return INSTANCE;
    }
    
    public VoxelCraiConfig getConfig() {
        return config;
    }
    
    public LightPatternBuffer getPatternBuffer() {
        return patternBuffer;
    }
    
    public PatternGenerator getPatternGenerator() {
        return patternGenerator;
    }
    
    public ShaderPackManager getShaderPackManager() {
        return shaderPackManager;
    }
    
    public boolean isInitialized() {
        return initialized;
    }
}
