package net.voxelcrai.mixin;

import net.minecraft.client.render.WorldRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.voxelcrai.mod.VoxelCraiMod;
import net.voxelcrai.pattern.LightPatternBuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * 🎭 WorldRendererMixin - Хук для обновления паттернов при рендере
 * 
 * Вставляет вызов обновления SSBO буфера перед рендером мира
 */
@Mixin(WorldRenderer.class)
public abstract class WorldRendererMixin {
    
    /**
     * 🔄 Хук перед рендером мира
     */
    @Inject(method = "render", at = @At("HEAD"))
    private void voxelcrai$onRenderStart(CallbackInfo ci) {
        VoxelCraiMod mod = VoxelCraiMod.getInstance();
        
        if (mod == null || !mod.isInitialized()) {
            return;
        }
        
        LightPatternBuffer buffer = mod.getPatternBuffer();
        
        // 🔄 Проверяем, нужно ли обновить GPU буфер
        if (buffer.isDirty()) {
            // Здесь будет вызов обновления SSBO через Iris API
            // Пока просто логируем для отладки
            if (mod.getConfig().isDebugMode()) {
                VoxelCraiMod.LOGGER.debug("🔄 Обновление GPU буфера: {} паттернов", 
                    buffer.getPatternCount());
            }
            
            buffer.clearDirty();
        }
    }
    
    /**
     * 🎬 Хук после рендера мира
     */
    @Inject(method = "render", at = @At("RETURN"))
    private void voxelcrai$onRenderEnd(CallbackInfo ci) {
        VoxelCraiMod mod = VoxelCraiMod.getInstance();
        
        if (mod == null || !mod.isInitialized()) {
            return;
        }
        
        // 📊 Отображение статистики (если включено)
        if (mod.getConfig().isShowPatternCount()) {
            LightPatternBuffer buffer = mod.getPatternBuffer();
            // Статистика будет отображаться через HUD
        }
    }
}
