package net.voxelcrai.mixin;

import net.minecraft.client.render.chunk.ChunkBuilder;
import net.voxelcrai.mod.VoxelCraiMod;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * 🧱 ChunkBuilderMixin - Хук для обновления паттернов при перестройке чанков
 * 
 * Триггерит генерацию паттернов при изменении блоков в чанке
 */
@Mixin(ChunkBuilder.class)
public abstract class ChunkBuilderMixin {
    
    /**
     * 🔄 Хук при завершении построения чанка
     */
    @Inject(method = "reset", at = @At("RETURN"))
    private void voxelcrai$onChunkReset(CallbackInfo ci) {
        VoxelCraiMod mod = VoxelCraiMod.getInstance();
        
        if (mod == null || !mod.isInitialized()) {
            return;
        }
        
        // 🔄 При сбросе ChunkBuilder может потребоваться обновление паттернов
        // Логика обновления вызывается через события ClientChunkEvents
        
        if (mod.getConfig().isDebugMode()) {
            VoxelCraiMod.LOGGER.debug("🧱 ChunkBuilder reset - паттерны будут обновлены");
        }
    }
}
