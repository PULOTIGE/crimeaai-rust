#version 330 core
// 🚀 VoxelCrai - Final Pass Fragment Shader
// Финальные эффекты: виньетка, цветокоррекция

in vec2 texCoord;

layout(location = 0) out vec4 outColor;

uniform sampler2D colortex0;  // Результат composite pass

void main() {
    vec3 color = texture(colortex0, texCoord).rgb;
    
    // 🎭 Легкая виньетка (опционально)
    // float vignette = 1.0 - length(texCoord - 0.5) * 0.5;
    // color *= vignette;
    
    // 🎨 Небольшое повышение контраста
    color = color * 1.05 - 0.025;
    color = clamp(color, 0.0, 1.0);
    
    outColor = vec4(color, 1.0);
}
