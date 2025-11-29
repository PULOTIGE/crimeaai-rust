// 🚀 VoxelCraiMod - Gradle Settings
// Воксельное освещение на основе Spherical Harmonics паттернов

pluginManagement {
    repositories {
        maven("https://maven.fabricmc.net/") {
            name = "Fabric"
        }
        mavenCentral()
        gradlePluginPortal()
    }
}

rootProject.name = "voxelcrai-mod"
