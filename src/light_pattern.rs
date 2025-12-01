//! # LightPattern - Паттерн освещения (1 КБ)
//!
//! Паттерн содержит данные освещения для быстрого рендеринга.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Свойства материала
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MaterialProps {
    pub roughness: f32,
    pub metalness: f32,
    pub albedo: [f32; 3],
    pub emission: [f32; 3],
}

impl Default for MaterialProps {
    fn default() -> Self {
        Self {
            roughness: 0.5,
            metalness: 0.0,
            albedo: [0.8, 0.8, 0.8],
            emission: [0.0, 0.0, 0.0],
        }
    }
}

/// Паттерн освещения - 1 КБ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightPattern {
    pub id: u32,
    pub direct_lighting: [[f32; 3]; 32],   // 32 источника x RGB
    pub indirect_lighting: [[f32; 3]; 32], // Отражённое
    pub sh_coeffs: [[f32; 3]; 9],          // Сферические гармоники
    pub material: MaterialProps,
    pub importance: f32,
    pub use_count: u32,
}

impl Default for LightPattern {
    fn default() -> Self {
        Self {
            id: 0,
            direct_lighting: [[0.0; 3]; 32],
            indirect_lighting: [[0.0; 3]; 32],
            sh_coeffs: [[0.0; 3]; 9],
            material: MaterialProps::default(),
            importance: 1.0,
            use_count: 0,
        }
    }
}

impl LightPattern {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        
        let mut pattern = Self::default();
        
        for i in 0..32 {
            pattern.direct_lighting[i] = [
                rng.gen_range(0.0..0.5),
                rng.gen_range(0.0..0.5),
                rng.gen_range(0.0..0.5),
            ];
            pattern.indirect_lighting[i] = [
                rng.gen_range(0.0..0.2),
                rng.gen_range(0.0..0.2),
                rng.gen_range(0.0..0.2),
            ];
        }
        
        for i in 0..9 {
            pattern.sh_coeffs[i] = [
                rng.gen_range(-0.1..0.1),
                rng.gen_range(-0.1..0.1),
                rng.gen_range(-0.1..0.1),
            ];
        }
        
        pattern.material = MaterialProps {
            roughness: rng.gen(),
            metalness: rng.gen_range(0.0..0.5),
            albedo: [rng.gen(), rng.gen(), rng.gen()],
            emission: [0.0; 3],
        };
        
        pattern.importance = rng.gen();
        
        pattern
    }
    
    /// Получение вектора признаков
    pub fn feature_vector(&self) -> Vec<f32> {
        let mut features = Vec::with_capacity(256);
        
        for d in &self.direct_lighting {
            features.extend_from_slice(d);
        }
        for i in &self.indirect_lighting {
            features.extend_from_slice(i);
        }
        for s in &self.sh_coeffs {
            features.extend_from_slice(s);
        }
        
        features.push(self.material.roughness);
        features.push(self.material.metalness);
        features.extend_from_slice(&self.material.albedo);
        
        features
    }
    
    /// Применение паттерна к позиции
    pub fn apply(&self, _position: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        // Упрощённая версия - берём среднее освещение
        let mut direct = [0.0f32; 3];
        let mut indirect = [0.0f32; 3];
        
        for i in 0..32 {
            for j in 0..3 {
                direct[j] += self.direct_lighting[i][j];
                indirect[j] += self.indirect_lighting[i][j];
            }
        }
        
        for j in 0..3 {
            direct[j] /= 32.0;
            indirect[j] /= 32.0;
        }
        
        (direct, indirect)
    }
    
    /// Смешивание с другим паттерном
    pub fn blend(&self, other: &LightPattern, weight: f32) -> LightPattern {
        let w1 = 1.0 - weight;
        let w2 = weight;
        
        let mut result = LightPattern::default();
        
        for i in 0..32 {
            for j in 0..3 {
                result.direct_lighting[i][j] = 
                    w1 * self.direct_lighting[i][j] + w2 * other.direct_lighting[i][j];
                result.indirect_lighting[i][j] = 
                    w1 * self.indirect_lighting[i][j] + w2 * other.indirect_lighting[i][j];
            }
        }
        
        for i in 0..9 {
            for j in 0..3 {
                result.sh_coeffs[i][j] = 
                    w1 * self.sh_coeffs[i][j] + w2 * other.sh_coeffs[i][j];
            }
        }
        
        result.material.roughness = w1 * self.material.roughness + w2 * other.material.roughness;
        result.material.metalness = w1 * self.material.metalness + w2 * other.material.metalness;
        
        result
    }
}

/// База данных паттернов
pub struct PatternDatabase {
    patterns: Vec<LightPattern>,
    max_patterns: usize,
    next_id: u32,
    pub total_lookups: u64,
}

impl PatternDatabase {
    pub fn new(max_patterns: usize) -> Self {
        Self {
            patterns: Vec::with_capacity(max_patterns),
            max_patterns,
            next_id: 0,
            total_lookups: 0,
        }
    }
    
    pub fn add(&mut self, mut pattern: LightPattern) -> u32 {
        if self.patterns.len() >= self.max_patterns {
            // Удаляем наименее используемый
            if let Some(min_idx) = self.patterns
                .iter()
                .enumerate()
                .min_by_key(|(_, p)| p.use_count)
                .map(|(i, _)| i)
            {
                self.patterns.remove(min_idx);
            }
        }
        
        let id = self.next_id;
        self.next_id += 1;
        pattern.id = id;
        self.patterns.push(pattern);
        
        id
    }
    
    pub fn generate_random(&mut self, count: usize) {
        for _ in 0..count {
            self.add(LightPattern::random());
        }
        println!("✨ Сгенерировано {} паттернов", count);
    }
    
    /// Поиск похожих паттернов
    pub fn find_similar(&mut self, query: &[f32], top_k: usize) -> Vec<(f32, &LightPattern)> {
        self.total_lookups += 1;
        
        let mut results: Vec<(f32, usize)> = self.patterns
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let features = p.feature_vector();
                let sim = cosine_similarity(query, &features);
                (sim, i)
            })
            .collect();
        
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        results.truncate(top_k);
        
        // Увеличиваем счётчик использования
        for &(_, idx) in &results {
            self.patterns[idx].use_count += 1;
        }
        
        results.into_iter()
            .map(|(sim, idx)| (sim, &self.patterns[idx]))
            .collect()
    }
    
    pub fn count(&self) -> usize {
        self.patterns.len()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    
    for i in 0..len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    let norm = (norm_a * norm_b).sqrt();
    if norm < 1e-8 {
        0.0
    } else {
        dot / norm
    }
}
