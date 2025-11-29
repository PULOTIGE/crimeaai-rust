package net.voxelcrai.mod;

import com.terraformersmc.modmenu.api.ConfigScreenFactory;
import com.terraformersmc.modmenu.api.ModMenuApi;
import net.minecraft.client.gui.screen.Screen;
import net.voxelcrai.config.VoxelCraiConfigScreen;

/**
 * 🎛️ Интеграция с Mod Menu
 * Предоставляет экран конфигурации для настройки паттернов
 */
public class VoxelCraiModMenu implements ModMenuApi {
    
    @Override
    public ConfigScreenFactory<?> getModConfigScreenFactory() {
        return VoxelCraiConfigScreen::new;
    }
}
