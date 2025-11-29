package net.voxelcrai.pattern;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/**
 * 💡 LightPattern1KB - Структура паттерна освещения (1024 байта)
 * 
 * Портировано из Rust прототипа:
 * ```rust
 * #[repr(C)]
 * struct LightPattern1KB {
 *     id: u64,              // 8B
 *     _pad0: [u8; 8],       // 8B (выравнивание)
 *     direct_light: [u16; 3],   // RGB fp16 direct (6B)
 *     indirect_light: [u16; 3], // RGB fp16 indirect (6B)
 *     sh_coeffs: [i8; 9],   // 3 bands SH для GI/shadows (9B)
 *     material_roughness: u8,   // 1B
 *     material_metallic: u8,    // 1B
 *     flags: u16,           // 2B
 *     _padding: [u8; 942],  // Reserve to 1KB
 * }
 * ```
 * 
 * 🔮 SH (Spherical Harmonics) коэффициенты:
 * - Band 0: 1 коэффициент (DC/ambient)
 * - Band 1: 3 коэффициента (направленный свет)
 * - Band 2: 5 коэффициентов (мягкие тени)
 * - Band 3: 7 коэффициентов (детали) - опционально
 * 
 * Расширенная версия с 16 SH коэффициентами для высокого качества
 */
public class LightPattern1KB {
    
    // 📏 Константы размеров
    public static final int SIZE_BYTES = 1024;
    public static final int SH_COEFFS_COUNT = 16;  // До 4 bands SH
    public static final int SH_COEFFS_EXTENDED = 256;  // Расширенные SH для детальных теней
    public static final int MATERIAL_DATA_SIZE = 512;
    
    // 🆔 Идентификатор паттерна (8 байт)
    private long id;
    
    // 💡 Прямое освещение RGB (fp16 - half-float, хранится как short)
    private short directR;
    private short directG;
    private short directB;
    
    // 🌙 Непрямое освещение RGB (fp16)
    private short indirectR;
    private short indirectG;
    private short indirectB;
    
    // 🔮 SH коэффициенты (i8, нормализованы в [-127, 127])
    // Расширенная версия: 256 коэффициентов для детальных теней
    private byte[] shCoefficients;
    
    // 🎨 Материалы (512 байт)
    private byte[] materialData;
    
    // 🔧 Параметры материала
    private float roughness;  // [0.0, 1.0]
    private float metallic;   // [0.0, 1.0]
    
    // 🌅 Ambient Occlusion (fp16)
    private short ambientOcclusion;
    
    // ✨ Отражения и преломления (fp16)
    private short reflection;
    private short refraction;
    
    // 🔥 Эмиссия (fp16)
    private short emission;
    
    // 🚩 Флаги паттерна
    private short flags;
    
    // 📍 Позиция в мире (для привязки к чанку)
    private int posX;
    private int posY;
    private int posZ;
    
    /**
     * 🏗️ Конструктор по умолчанию
     */
    public LightPattern1KB() {
        this.id = 0;
        this.directR = 0;
        this.directG = 0;
        this.directB = 0;
        this.indirectR = 0;
        this.indirectG = 0;
        this.indirectB = 0;
        this.shCoefficients = new byte[SH_COEFFS_EXTENDED];
        this.materialData = new byte[MATERIAL_DATA_SIZE];
        this.roughness = 0.5f;
        this.metallic = 0.0f;
        this.ambientOcclusion = floatToHalf(1.0f);
        this.reflection = floatToHalf(0.0f);
        this.refraction = floatToHalf(0.0f);
        this.emission = floatToHalf(0.0f);
        this.flags = 0;
        this.posX = 0;
        this.posY = 0;
        this.posZ = 0;
    }
    
    /**
     * 🏗️ Конструктор с ID
     */
    public LightPattern1KB(long id) {
        this();
        this.id = id;
    }
    
    /**
     * 🔮 Установка SH коэффициента
     * 
     * @param index индекс коэффициента [0-255]
     * @param value значение [-127, 127]
     */
    public void setShCoefficient(int index, byte value) {
        if (index >= 0 && index < SH_COEFFS_EXTENDED) {
            shCoefficients[index] = value;
        }
    }
    
    /**
     * 🔮 Получение SH коэффициента
     */
    public byte getShCoefficient(int index) {
        if (index >= 0 && index < SH_COEFFS_EXTENDED) {
            return shCoefficients[index];
        }
        return 0;
    }
    
    /**
     * 💡 Установка прямого освещения (RGB float -> fp16)
     */
    public void setDirectLight(float r, float g, float b) {
        this.directR = floatToHalf(r);
        this.directG = floatToHalf(g);
        this.directB = floatToHalf(b);
    }
    
    /**
     * 🌙 Установка непрямого освещения (RGB float -> fp16)
     */
    public void setIndirectLight(float r, float g, float b) {
        this.indirectR = floatToHalf(r);
        this.indirectG = floatToHalf(g);
        this.indirectB = floatToHalf(b);
    }
    
    /**
     * 🔮 Установка SH коэффициентов для 3-х bands (9 коэффициентов)
     * 
     * Band 0: Y00 = 0.282095 (DC)
     * Band 1: Y1-1, Y10, Y11 (направленный свет)
     * Band 2: Y2-2, Y2-1, Y20, Y21, Y22 (мягкие тени)
     */
    public void setShCoefficients3Bands(byte[] coeffs) {
        if (coeffs.length >= 9) {
            System.arraycopy(coeffs, 0, shCoefficients, 0, 9);
        }
    }
    
    /**
     * 🔮 Установка SH коэффициентов для 4-х bands (16 коэффициентов)
     */
    public void setShCoefficients4Bands(byte[] coeffs) {
        if (coeffs.length >= 16) {
            System.arraycopy(coeffs, 0, shCoefficients, 0, 16);
        }
    }
    
    /**
     * 📦 Сериализация в ByteBuffer для SSBO
     * 
     * Формат GPU-совместимый (std430 layout):
     * - 8B: id (uvec2)
     * - 8B: padding
     * - 6B: direct RGB (3x fp16)
     * - 6B: indirect RGB (3x fp16)
     * - 256B: SH coefficients
     * - 512B: material data
     * - 4B: roughness/metallic (2x fp16)
     * - 8B: AO/reflection/refraction/emission (4x fp16)
     * - 2B: flags
     * - 12B: position (3x i32)
     * - padding до 1024B
     */
    public ByteBuffer toByteBuffer() {
        ByteBuffer buffer = ByteBuffer.allocate(SIZE_BYTES);
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        
        // 🆔 ID (8 байт)
        buffer.putLong(id);
        
        // Padding (8 байт)
        buffer.putLong(0);
        
        // 💡 Direct light RGB fp16 (6 байт)
        buffer.putShort(directR);
        buffer.putShort(directG);
        buffer.putShort(directB);
        
        // 🌙 Indirect light RGB fp16 (6 байт)
        buffer.putShort(indirectR);
        buffer.putShort(indirectG);
        buffer.putShort(indirectB);
        
        // 🔮 SH коэффициенты (256 байт)
        buffer.put(shCoefficients);
        
        // 🎨 Material data (512 байт)
        buffer.put(materialData);
        
        // 🔧 Roughness/Metallic (4 байта)
        buffer.putShort(floatToHalf(roughness));
        buffer.putShort(floatToHalf(metallic));
        
        // ✨ AO/Reflection/Refraction/Emission (8 байт)
        buffer.putShort(ambientOcclusion);
        buffer.putShort(reflection);
        buffer.putShort(refraction);
        buffer.putShort(emission);
        
        // 🚩 Flags (2 байта)
        buffer.putShort(flags);
        
        // 📍 Position (12 байт)
        buffer.putInt(posX);
        buffer.putInt(posY);
        buffer.putInt(posZ);
        
        // Padding до 1024 байт
        // Уже использовано: 8+8+6+6+256+512+4+8+2+12 = 822 байт
        // Нужно добавить: 1024-822 = 202 байт padding
        byte[] padding = new byte[202];
        buffer.put(padding);
        
        buffer.flip();
        return buffer;
    }
    
    /**
     * 📦 Десериализация из ByteBuffer
     */
    public static LightPattern1KB fromByteBuffer(ByteBuffer buffer) {
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        
        LightPattern1KB pattern = new LightPattern1KB();
        
        pattern.id = buffer.getLong();
        buffer.getLong(); // Skip padding
        
        pattern.directR = buffer.getShort();
        pattern.directG = buffer.getShort();
        pattern.directB = buffer.getShort();
        
        pattern.indirectR = buffer.getShort();
        pattern.indirectG = buffer.getShort();
        pattern.indirectB = buffer.getShort();
        
        buffer.get(pattern.shCoefficients);
        buffer.get(pattern.materialData);
        
        pattern.roughness = halfToFloat(buffer.getShort());
        pattern.metallic = halfToFloat(buffer.getShort());
        
        pattern.ambientOcclusion = buffer.getShort();
        pattern.reflection = buffer.getShort();
        pattern.refraction = buffer.getShort();
        pattern.emission = buffer.getShort();
        
        pattern.flags = buffer.getShort();
        
        pattern.posX = buffer.getInt();
        pattern.posY = buffer.getInt();
        pattern.posZ = buffer.getInt();
        
        return pattern;
    }
    
    // ========== 🔢 Вспомогательные функции FP16 ==========
    
    /**
     * 🔢 Конвертация float -> half-precision (fp16)
     * IEEE 754 half-precision binary floating-point format
     */
    public static short floatToHalf(float value) {
        int fbits = Float.floatToIntBits(value);
        int sign = (fbits >>> 16) & 0x8000;
        int val = (fbits & 0x7fffffff) + 0x1000;
        
        if (val >= 0x47800000) {
            // Overflow -> infinity
            if ((fbits & 0x7fffffff) >= 0x47800000) {
                if (val < 0x7f800000) {
                    return (short) (sign | 0x7c00);
                }
                return (short) (sign | 0x7c00 | ((fbits & 0x007fffff) >>> 13));
            }
            return (short) (sign | 0x7bff);
        }
        
        if (val >= 0x38800000) {
            return (short) (sign | ((val - 0x38000000) >>> 13));
        }
        
        if (val < 0x33000000) {
            return (short) sign;
        }
        
        val = (fbits & 0x7fffffff) >>> 23;
        return (short) (sign | (((fbits & 0x7fffff) | 0x800000) + (0x800000 >>> (val - 102))) >>> (126 - val));
    }
    
    /**
     * 🔢 Конвертация half-precision (fp16) -> float
     */
    public static float halfToFloat(short half) {
        int mant = half & 0x03ff;
        int exp = half & 0x7c00;
        
        if (exp == 0x7c00) {
            exp = 0x3fc00;
        } else if (exp != 0) {
            exp += 0x1c000;
            if (mant == 0 && exp > 0x1c400) {
                return Float.intBitsToFloat((half & 0x8000) << 16 | exp << 13 | 0x3ff);
            }
        } else if (mant != 0) {
            exp = 0x1c400;
            do {
                mant <<= 1;
                exp -= 0x400;
            } while ((mant & 0x400) == 0);
            mant &= 0x3ff;
        }
        
        return Float.intBitsToFloat((half & 0x8000) << 16 | (exp | mant) << 13);
    }
    
    // ========== 🔧 Геттеры и сеттеры ==========
    
    public long getId() { return id; }
    public void setId(long id) { this.id = id; }
    
    public float getRoughness() { return roughness; }
    public void setRoughness(float roughness) { this.roughness = Math.max(0, Math.min(1, roughness)); }
    
    public float getMetallic() { return metallic; }
    public void setMetallic(float metallic) { this.metallic = Math.max(0, Math.min(1, metallic)); }
    
    public short getFlags() { return flags; }
    public void setFlags(short flags) { this.flags = flags; }
    
    public void setPosition(int x, int y, int z) {
        this.posX = x;
        this.posY = y;
        this.posZ = z;
    }
    
    public int getPosX() { return posX; }
    public int getPosY() { return posY; }
    public int getPosZ() { return posZ; }
    
    public void setAmbientOcclusion(float ao) { this.ambientOcclusion = floatToHalf(ao); }
    public void setReflection(float r) { this.reflection = floatToHalf(r); }
    public void setRefraction(float r) { this.refraction = floatToHalf(r); }
    public void setEmission(float e) { this.emission = floatToHalf(e); }
    
    public byte[] getShCoefficients() { return shCoefficients; }
    public byte[] getMaterialData() { return materialData; }
    
    /**
     * 📊 Получение размера в байтах
     */
    public static int getSizeBytes() {
        return SIZE_BYTES;
    }
    
    @Override
    public String toString() {
        return String.format("LightPattern1KB[id=%d, pos=(%d,%d,%d), roughness=%.2f, metallic=%.2f]",
            id, posX, posY, posZ, roughness, metallic);
    }
}
