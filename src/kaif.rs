//! # Kaif Engine - Движок эмоционального состояния
//!
//! Кайф = |dS/dt| - абсолютная величина производной энтропии

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Состояния кайфа
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaifState {
    Dormant,   // < 0.1
    Calm,      // 0.1 - 0.3
    Active,    // 0.3 - 0.6
    Excited,   // 0.6 - 0.8
    Ecstatic,  // > 0.8
}

impl KaifState {
    pub fn from_kaif(kaif: f32) -> Self {
        if kaif < 0.1 {
            Self::Dormant
        } else if kaif < 0.3 {
            Self::Calm
        } else if kaif < 0.6 {
            Self::Active
        } else if kaif < 0.8 {
            Self::Excited
        } else {
            Self::Ecstatic
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::Calm => "calm",
            Self::Active => "active",
            Self::Excited => "excited",
            Self::Ecstatic => "ecstatic",
        }
    }
    
    pub fn color(&self) -> [u8; 3] {
        match self {
            Self::Dormant => [60, 60, 80],
            Self::Calm => [100, 150, 200],
            Self::Active => [150, 200, 100],
            Self::Excited => [255, 180, 50],
            Self::Ecstatic => [255, 100, 200],
        }
    }
}

/// Вычисление энтропии Шеннона
pub fn compute_entropy(data: &[f32]) -> f32 {
    let sum: f32 = data.iter().map(|x| x.abs()).sum();
    if sum < 1e-10 {
        return 0.0;
    }
    
    let mut entropy = 0.0f32;
    for &v in data {
        let p = v.abs() / sum;
        if p > 1e-10 {
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

/// Метрики кайфа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaifMetrics {
    pub instant: f32,
    pub smoothed: f32,
    pub max_ever: f32,
    pub average: f32,
    pub variance: f32,
    pub state: KaifState,
    history: Vec<f32>,
    history_size: usize,
}

impl Default for KaifMetrics {
    fn default() -> Self {
        Self {
            instant: 0.0,
            smoothed: 0.0,
            max_ever: 0.0,
            average: 0.0,
            variance: 0.0,
            state: KaifState::Calm,
            history: Vec::with_capacity(100),
            history_size: 100,
        }
    }
}

impl KaifMetrics {
    pub fn update(&mut self, new_kaif: f32, smoothing: f32) {
        self.instant = new_kaif;
        self.smoothed = (1.0 - smoothing) * self.smoothed + smoothing * new_kaif;
        self.max_ever = self.max_ever.max(new_kaif);
        
        // История
        self.history.push(new_kaif);
        if self.history.len() > self.history_size {
            self.history.remove(0);
        }
        
        // Статистика
        if !self.history.is_empty() {
            self.average = self.history.iter().sum::<f32>() / self.history.len() as f32;
            self.variance = self.history.iter()
                .map(|&x| (x - self.average).powi(2))
                .sum::<f32>() / self.history.len() as f32;
        }
        
        // Состояние
        self.state = KaifState::from_kaif(self.smoothed);
    }
    
    pub fn trend(&self) -> &'static str {
        if self.history.len() < 10 {
            return "stable";
        }
        
        let recent: f32 = self.history[self.history.len()-10..].iter().sum::<f32>() / 10.0;
        let older: f32 = if self.history.len() >= 20 {
            self.history[self.history.len()-20..self.history.len()-10].iter().sum::<f32>() / 10.0
        } else {
            self.average
        };
        
        let diff = recent - older;
        if diff > 0.1 {
            "rising"
        } else if diff < -0.1 {
            "falling"
        } else {
            "stable"
        }
    }
}

/// Компонент для отслеживания
struct Component {
    current: Vec<f32>,
    previous: Vec<f32>,
    weight: f32,
}

/// Движок кайфа
pub struct KaifEngine {
    pub metrics: KaifMetrics,
    components: HashMap<String, Component>,
    pub update_count: u64,
}

impl KaifEngine {
    pub fn new() -> Self {
        Self {
            metrics: KaifMetrics::default(),
            components: HashMap::new(),
            update_count: 0,
        }
    }
    
    /// Регистрация компонента
    pub fn register_component(&mut self, name: &str, initial: Vec<f32>, weight: f32) {
        self.components.insert(name.to_string(), Component {
            current: initial.clone(),
            previous: initial,
            weight,
        });
    }
    
    /// Обновление компонента
    pub fn update_component(&mut self, name: &str, new_state: Vec<f32>) {
        if let Some(comp) = self.components.get_mut(name) {
            comp.previous = std::mem::replace(&mut comp.current, new_state);
        }
    }
    
    /// Вычисление общего кайфа
    pub fn compute_total_kaif(&self, dt: f32) -> f32 {
        if dt <= 0.0 || self.components.is_empty() {
            return 0.0;
        }
        
        let mut total_kaif = 0.0f32;
        let mut total_weight = 0.0f32;
        
        for comp in self.components.values() {
            let entropy_current = compute_entropy(&comp.current);
            let entropy_prev = compute_entropy(&comp.previous);
            let d_entropy = (entropy_current - entropy_prev) / dt;
            
            total_kaif += d_entropy.abs() * comp.weight;
            total_weight += comp.weight;
        }
        
        if total_weight > 0.0 {
            total_kaif / total_weight
        } else {
            0.0
        }
    }
    
    /// Главное обновление
    pub fn update(&mut self, dt: f32) {
        self.update_count += 1;
        let kaif = self.compute_total_kaif(dt);
        self.metrics.update(kaif, 0.1);
    }
    
    pub fn get_state(&self) -> KaifState {
        self.metrics.state
    }
    
    pub fn get_kaif(&self) -> f32 {
        self.metrics.smoothed
    }
    
    /// Инъекция стимула
    pub fn inject_stimulus(&mut self, intensity: f32) {
        let mut rng = rand::thread_rng();
        use rand::Rng;
        
        for comp in self.components.values_mut() {
            for v in &mut comp.current {
                *v += rng.gen_range(-intensity..intensity) * 0.5;
            }
        }
    }
}

impl Default for KaifEngine {
    fn default() -> Self {
        Self::new()
    }
}
