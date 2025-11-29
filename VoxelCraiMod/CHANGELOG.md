# 📜 VoxelCraiMod Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-XX-XX

### 🚀 Initial Release

#### Added
- **LightPattern1KB structure** - 1024-byte lighting pattern with:
  - 8-byte ID
  - RGB fp16 direct/indirect lighting
  - 256-byte SH coefficients (up to 4 bands)
  - 512-byte material data
  - Roughness, metallic, AO, reflection, refraction, emission parameters

- **Pattern Generator** - CPU-based pattern generation:
  - Fibonacci sphere sampling (64 directions)
  - SH projection for GI and shadows
  - Material detection from block type
  - Async chunk processing

- **GLSL Shader Pack** - Iris-compatible shader pack:
  - SH evaluation library (`lib/sh_eval.glsl`)
  - Terrain shaders with SH lighting
  - Composite pass for final composition
  - ACES tonemapping

- **Configuration System**:
  - Pattern count slider (1k-100k)
  - SH bands selector (3-5)
  - GI/Shadow/Reflection intensity controls
  - Mod Menu integration

- **Performance Optimizations**:
  - Spatial hashing for pattern lookup
  - SSBO buffer for GPU pattern storage
  - Async pattern generation
  - Configurable pattern density

#### Technical Details
- **SH Bands Support**:
  - 3 bands (9 coefficients) - Fast mode
  - 4 bands (16 coefficients) - Balanced mode
  - 5 bands (25 coefficients) - Quality mode

- **Memory Usage**:
  - 1KB per pattern
  - 10k patterns = 10 MB VRAM
  - Configurable up to 100k patterns (100 MB)

### 🎯 Target Performance
- 60+ FPS on AMD Radeon VII at 1080p
- 45+ FPS on NVIDIA RTX 2060 at 1080p
- Scales down to 30+ FPS on older GPUs

### 📦 Dependencies
- Minecraft 1.21.3+
- Fabric Loader 0.16.7+
- Fabric API
- (Optional) Iris Shaders 1.7.0+
- (Optional) Sodium 0.5.0+

---

## Future Plans

### [1.1.0] - Planned
- [ ] Color bleeding for colored GI
- [ ] Temporal accumulation for smoother lighting
- [ ] Better material detection (ore glow, water caustics)
- [ ] Block change notifications for pattern updates

### [1.2.0] - Planned
- [ ] Extended SH (5 bands) for better shadow detail
- [ ] Night vision / flashlight integration
- [ ] Weather-aware lighting (rain puddles)
- [ ] Performance profiler integration

---

*Made with ❤️ and Spherical Harmonics*
