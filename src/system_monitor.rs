use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Система мониторинга ресурсов
pub struct SystemMonitor {
    pub ram_used: Arc<AtomicU64>,      // В MB
    pub ram_total: Arc<AtomicU64>,     // В MB
    pub cpu_usage: Arc<AtomicU64>,     // В процентах
    pub vram_used: Arc<AtomicU64>,     // В MB
    pub vram_total: Arc<AtomicU64>,    // В MB
    pub fps: Arc<AtomicU64>,           // FPS (x100 для точности)
}

impl SystemMonitor {
    pub fn new() -> Self {
        let monitor = Self {
            ram_used: Arc::new(AtomicU64::new(0)),
            ram_total: Arc::new(AtomicU64::new(0)),
            cpu_usage: Arc::new(AtomicU64::new(0)),
            vram_used: Arc::new(AtomicU64::new(0)),
            vram_total: Arc::new(AtomicU64::new(0)),
            fps: Arc::new(AtomicU64::new(0)),
        };
        
        // Инициализируем начальные значения
        monitor.update_ram();
        monitor.update_cpu();
        
        monitor
    }
    
    /// Обновить информацию о RAM
    pub fn update_ram(&self) {
        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
            
            unsafe {
                let mut mem_info: MEMORYSTATUSEX = mem::zeroed();
                mem_info.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
                
                if GlobalMemoryStatusEx(&mut mem_info) != 0 {
                    let total_mb = (mem_info.ullTotalPhys / 1024 / 1024) as u64;
                    let available_mb = (mem_info.ullAvailPhys / 1024 / 1024) as u64;
                    let used_mb = total_mb.saturating_sub(available_mb);
                    
                    self.ram_total.store(total_mb, Ordering::Relaxed);
                    self.ram_used.store(used_mb, Ordering::Relaxed);
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Linux/Mac: используем sysinfo или примерные значения
            // Для простоты используем фиксированные значения
            self.ram_total.store(16384, Ordering::Relaxed); // 16 GB
            self.ram_used.store(4096, Ordering::Relaxed);   // 4 GB
        }
    }
    
    /// Обновить информацию о CPU
    pub fn update_cpu(&self) {
        // Простая имитация загрузки CPU
        // В реальном приложении использовать sysinfo crate
        let usage = 15 + (rand::random::<u64>() % 30); // 15-45%
        self.cpu_usage.store(usage, Ordering::Relaxed);
    }
    
    /// Обновить информацию о VRAM (примерные значения)
    pub fn update_vram(&self, voxel_count: usize) {
        // Примерный расчет: каждый воксель ~10 KB
        let used_mb = (voxel_count * 10) / 1024;
        self.vram_used.store(used_mb as u64, Ordering::Relaxed);
        
        // Общий VRAM (примерно)
        self.vram_total.store(4096, Ordering::Relaxed); // 4 GB
    }
    
    /// Обновить FPS
    pub fn update_fps(&self, fps: f32) {
        // Храним FPS * 100 для точности
        let fps_x100 = (fps * 100.0) as u64;
        self.fps.store(fps_x100, Ordering::Relaxed);
    }
    
    /// Получить использование RAM в процентах
    pub fn get_ram_percent(&self) -> f32 {
        let used = self.ram_used.load(Ordering::Relaxed) as f32;
        let total = self.ram_total.load(Ordering::Relaxed) as f32;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
    
    /// Получить использование VRAM в процентах
    pub fn get_vram_percent(&self) -> f32 {
        let used = self.vram_used.load(Ordering::Relaxed) as f32;
        let total = self.vram_total.load(Ordering::Relaxed) as f32;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
    
    /// Получить FPS
    pub fn get_fps(&self) -> f32 {
        let fps_x100 = self.fps.load(Ordering::Relaxed);
        (fps_x100 as f32) / 100.0
    }
    
    /// Форматировать байты в человекочитаемый вид
    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f32 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f32 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f32 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Заглушка для rand без зависимости
mod rand {
    pub fn random<T>() -> T 
    where 
        T: From<u64>
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        T::from((nanos % 100) as u64)
    }
}
