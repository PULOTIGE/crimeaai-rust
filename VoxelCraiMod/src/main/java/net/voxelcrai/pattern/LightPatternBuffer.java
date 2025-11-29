package net.voxelcrai.pattern;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.locks.ReentrantReadWriteLock;

/**
 * 💾 LightPatternBuffer - Буфер паттернов для GPU SSBO
 * 
 * Управляет коллекцией LightPattern1KB для передачи в шейдеры.
 * Thread-safe для асинхронной генерации паттернов.
 * 
 * Размеры:
 * - 1k паттернов = 1 MB
 * - 10k паттернов = 10 MB
 * - Max 100k паттернов = 100 MB (для больших миров)
 */
public class LightPatternBuffer {
    
    // 📏 Константы
    public static final int DEFAULT_CAPACITY = 10_000;  // 10k паттернов по умолчанию
    public static final int MIN_CAPACITY = 1_000;       // Минимум 1k
    public static final int MAX_CAPACITY = 100_000;     // Максимум 100k
    
    // 💾 Хранилище паттернов
    private final ConcurrentHashMap<Long, LightPattern1KB> patterns;
    private final List<LightPattern1KB> orderedPatterns;
    private final ReentrantReadWriteLock lock;
    
    // 📊 Метаданные буфера
    private int capacity;
    private volatile boolean dirty;
    private volatile long lastUpdateTime;
    
    // 📦 GPU буфер (lazy initialization)
    private ByteBuffer gpuBuffer;
    private volatile boolean gpuBufferDirty;
    
    /**
     * 🏗️ Конструктор с емкостью по умолчанию
     */
    public LightPatternBuffer() {
        this(DEFAULT_CAPACITY);
    }
    
    /**
     * 🏗️ Конструктор с заданной емкостью
     */
    public LightPatternBuffer(int capacity) {
        this.capacity = Math.max(MIN_CAPACITY, Math.min(MAX_CAPACITY, capacity));
        this.patterns = new ConcurrentHashMap<>(capacity);
        this.orderedPatterns = new ArrayList<>(capacity);
        this.lock = new ReentrantReadWriteLock();
        this.dirty = false;
        this.lastUpdateTime = System.currentTimeMillis();
        this.gpuBufferDirty = true;
    }
    
    /**
     * ➕ Добавление паттерна
     */
    public void addPattern(LightPattern1KB pattern) {
        lock.writeLock().lock();
        try {
            if (orderedPatterns.size() >= capacity) {
                // 🗑️ Удаляем самый старый паттерн
                LightPattern1KB oldest = orderedPatterns.remove(0);
                patterns.remove(oldest.getId());
            }
            
            patterns.put(pattern.getId(), pattern);
            orderedPatterns.add(pattern);
            markDirty();
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 🔄 Обновление паттерна
     */
    public void updatePattern(LightPattern1KB pattern) {
        lock.writeLock().lock();
        try {
            LightPattern1KB existing = patterns.get(pattern.getId());
            if (existing != null) {
                int index = orderedPatterns.indexOf(existing);
                if (index >= 0) {
                    orderedPatterns.set(index, pattern);
                }
            }
            patterns.put(pattern.getId(), pattern);
            markDirty();
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 📦 Массовое обновление паттернов
     */
    public void updatePatterns(List<LightPattern1KB> newPatterns) {
        lock.writeLock().lock();
        try {
            for (LightPattern1KB pattern : newPatterns) {
                LightPattern1KB existing = patterns.get(pattern.getId());
                if (existing != null) {
                    int index = orderedPatterns.indexOf(existing);
                    if (index >= 0) {
                        orderedPatterns.set(index, pattern);
                    }
                    patterns.put(pattern.getId(), pattern);
                } else if (orderedPatterns.size() < capacity) {
                    patterns.put(pattern.getId(), pattern);
                    orderedPatterns.add(pattern);
                }
            }
            markDirty();
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 🗑️ Удаление паттерна по ID
     */
    public void removePattern(long id) {
        lock.writeLock().lock();
        try {
            LightPattern1KB removed = patterns.remove(id);
            if (removed != null) {
                orderedPatterns.remove(removed);
                markDirty();
            }
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 🔍 Получение паттерна по ID
     */
    public LightPattern1KB getPattern(long id) {
        return patterns.get(id);
    }
    
    /**
     * 🔍 Получение паттерна по индексу
     */
    public LightPattern1KB getPatternByIndex(int index) {
        lock.readLock().lock();
        try {
            if (index >= 0 && index < orderedPatterns.size()) {
                return orderedPatterns.get(index);
            }
            return null;
        } finally {
            lock.readLock().unlock();
        }
    }
    
    /**
     * 📊 Получение количества паттернов
     */
    public int getPatternCount() {
        return orderedPatterns.size();
    }
    
    /**
     * 📊 Получение размера в KB
     */
    public int getSizeKB() {
        return (orderedPatterns.size() * LightPattern1KB.SIZE_BYTES) / 1024;
    }
    
    /**
     * 📊 Получение размера в MB
     */
    public float getSizeMB() {
        return getSizeKB() / 1024.0f;
    }
    
    /**
     * 📦 Получение GPU буфера (ByteBuffer для SSBO)
     */
    public ByteBuffer getGpuBuffer() {
        if (gpuBufferDirty || gpuBuffer == null) {
            rebuildGpuBuffer();
        }
        return gpuBuffer;
    }
    
    /**
     * 🔄 Пересборка GPU буфера
     */
    private void rebuildGpuBuffer() {
        lock.readLock().lock();
        try {
            int bufferSize = orderedPatterns.size() * LightPattern1KB.SIZE_BYTES;
            
            if (gpuBuffer == null || gpuBuffer.capacity() != bufferSize) {
                gpuBuffer = ByteBuffer.allocateDirect(bufferSize);
                gpuBuffer.order(ByteOrder.LITTLE_ENDIAN);
            }
            
            gpuBuffer.clear();
            
            for (LightPattern1KB pattern : orderedPatterns) {
                ByteBuffer patternBuffer = pattern.toByteBuffer();
                gpuBuffer.put(patternBuffer);
            }
            
            gpuBuffer.flip();
            gpuBufferDirty = false;
        } finally {
            lock.readLock().unlock();
        }
    }
    
    /**
     * 🗑️ Очистка всех паттернов
     */
    public void clear() {
        lock.writeLock().lock();
        try {
            patterns.clear();
            orderedPatterns.clear();
            markDirty();
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 🚩 Пометка буфера как измененного
     */
    private void markDirty() {
        dirty = true;
        gpuBufferDirty = true;
        lastUpdateTime = System.currentTimeMillis();
    }
    
    /**
     * ✅ Проверка, изменен ли буфер
     */
    public boolean isDirty() {
        return dirty;
    }
    
    /**
     * ✅ Сброс флага dirty
     */
    public void clearDirty() {
        dirty = false;
    }
    
    /**
     * ⏰ Получение времени последнего обновления
     */
    public long getLastUpdateTime() {
        return lastUpdateTime;
    }
    
    /**
     * 📊 Получение емкости буфера
     */
    public int getCapacity() {
        return capacity;
    }
    
    /**
     * 🔧 Изменение емкости буфера
     */
    public void setCapacity(int newCapacity) {
        lock.writeLock().lock();
        try {
            this.capacity = Math.max(MIN_CAPACITY, Math.min(MAX_CAPACITY, newCapacity));
            
            // Обрезаем, если нужно
            while (orderedPatterns.size() > capacity) {
                LightPattern1KB removed = orderedPatterns.remove(0);
                patterns.remove(removed.getId());
            }
            
            markDirty();
        } finally {
            lock.writeLock().unlock();
        }
    }
    
    /**
     * 📋 Получение копии всех паттернов
     */
    public List<LightPattern1KB> getAllPatterns() {
        lock.readLock().lock();
        try {
            return new ArrayList<>(orderedPatterns);
        } finally {
            lock.readLock().unlock();
        }
    }
    
    @Override
    public String toString() {
        return String.format("LightPatternBuffer[count=%d, capacity=%d, size=%.2f MB, dirty=%s]",
            getPatternCount(), capacity, getSizeMB(), dirty);
    }
}
