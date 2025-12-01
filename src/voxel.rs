//! # Voxel - Микро-организм (9 КБ)
//!
//! Воксель - автономная сущность с сенсорами, физикой, мыслями, эмоциями и памятью.
//!
//! ## Структура (9216 байт = 9 КБ):
//! - 512 Б: metadata
//! - 1536 Б (1.5 КБ): sensors
//! - 1024 Б (1 КБ): physics  
//! - 2048 Б (2 КБ): thoughts
//! - 2048 Б (2 КБ): emotions
//! - 2048 Б (2 КБ): memory

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Типы эмоций
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionType {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Curiosity,
    Peace,
}

impl EmotionType {
    pub fn all() -> [EmotionType; 8] {
        [
            Self::Joy, Self::Sadness, Self::Anger, Self::Fear,
            Self::Surprise, Self::Disgust, Self::Curiosity, Self::Peace,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Joy => "joy",
            Self::Sadness => "sadness",
            Self::Anger => "anger",
            Self::Fear => "fear",
            Self::Surprise => "surprise",
            Self::Disgust => "disgust",
            Self::Curiosity => "curiosity",
            Self::Peace => "peace",
        }
    }
    
    pub fn color(&self) -> [u8; 3] {
        match self {
            Self::Joy => [255, 220, 50],
            Self::Sadness => [80, 100, 200],
            Self::Anger => [255, 50, 50],
            Self::Fear => [180, 100, 255],
            Self::Surprise => [255, 150, 0],
            Self::Disgust => [100, 180, 80],
            Self::Curiosity => [0, 220, 255],
            Self::Peace => [200, 200, 220],
        }
    }
}

/// Метаданные вокселя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelMetadata {
    pub id: u64,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub temperature: f32,
    pub age_ticks: u64,
    pub health: f32,
    pub energy: f32,
    pub state: u8, // 0=normal, 1=active, 2=dormant
}

impl Default for VoxelMetadata {
    fn default() -> Self {
        Self {
            id: 0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            mass: 1.0,
            temperature: 300.0,
            age_ticks: 0,
            health: 1.0,
            energy: 1.0,
            state: 0,
        }
    }
}

/// Сенсоры вокселя
#[derive(Debug, Clone)]
pub struct VoxelSensors {
    pub visual: [[f32; 3]; 32],    // 32 направления x RGB
    pub audio: [f32; 64],           // 64 частотных канала
    pub tactile: [[f32; 16]; 6],    // 6 сторон x 16 точек
    pub chemical: [f32; 32],        // 32 химических сенсора
    pub thermal: [f32; 8],          // 8 температурных
}

impl Default for VoxelSensors {
    fn default() -> Self {
        Self {
            visual: [[0.0; 3]; 32],
            audio: [0.0; 64],
            tactile: [[0.0; 16]; 6],
            chemical: [0.0; 32],
            thermal: [0.0; 8],
        }
    }
}

impl VoxelSensors {
    /// Получение объединённого сенсорного вектора
    pub fn combined(&self) -> Vec<f32> {
        let mut result = Vec::with_capacity(384);
        
        for v in &self.visual {
            result.extend_from_slice(v);
        }
        result.extend_from_slice(&self.audio);
        for t in &self.tactile {
            result.extend_from_slice(t);
        }
        result.extend_from_slice(&self.chemical);
        result.extend_from_slice(&self.thermal);
        
        result
    }
}

/// Физика вокселя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelPhysics {
    pub angular_velocity: [f32; 3],
    pub orientation: [f32; 4], // Кватернион
    pub accumulated_force: [f32; 3],
    pub accumulated_torque: [f32; 3],
    pub elasticity: f32,
    pub friction: f32,
}

impl Default for VoxelPhysics {
    fn default() -> Self {
        Self {
            angular_velocity: [0.0; 3],
            orientation: [1.0, 0.0, 0.0, 0.0],
            accumulated_force: [0.0; 3],
            accumulated_torque: [0.0; 3],
            elasticity: 0.5,
            friction: 0.3,
        }
    }
}

impl VoxelPhysics {
    pub fn apply_force(&mut self, force: [f32; 3]) {
        for i in 0..3 {
            self.accumulated_force[i] += force[i];
        }
    }
    
    pub fn integrate(&mut self, dt: f32, mass: f32) -> [f32; 3] {
        let mut acceleration = [0.0f32; 3];
        for i in 0..3 {
            acceleration[i] = self.accumulated_force[i] / mass;
            self.angular_velocity[i] += self.accumulated_torque[i] / mass * dt;
            self.angular_velocity[i] *= 0.99; // Затухание
        }
        
        // Сброс
        self.accumulated_force = [0.0; 3];
        self.accumulated_torque = [0.0; 3];
        
        acceleration
    }
}

/// Мысли вокселя
#[derive(Debug, Clone)]
pub struct VoxelThoughts {
    pub attention_focus: [f32; 128],
    pub working_memory: [f32; 256],
    pub processing_depth: u8,
}

impl Default for VoxelThoughts {
    fn default() -> Self {
        Self {
            attention_focus: [0.0; 128],
            working_memory: [0.0; 256],
            processing_depth: 0,
        }
    }
}

impl VoxelThoughts {
    pub fn process(&mut self, sensory_input: &[f32], dt: f32) {
        // Обновляем фокус внимания
        let len = sensory_input.len().min(128);
        for i in 0..len {
            self.attention_focus[i] = 0.9 * self.attention_focus[i] + 0.1 * sensory_input[i];
        }
        
        // Обновляем рабочую память
        for i in 0..128 {
            self.working_memory[i] = 0.95 * self.working_memory[i] + 0.05 * self.attention_focus[i];
            self.working_memory[128 + i] = 0.95 * self.working_memory[128 + i] + 0.05 * self.attention_focus[i];
        }
    }
}

/// Эмоции вокселя
#[derive(Debug, Clone)]
pub struct VoxelEmotions {
    pub base_emotions: [f32; 8], // 8 базовых эмоций
    pub emotion_vector: [f32; 256],
    pub kaif: f32,
    prev_entropy: f32,
}

impl Default for VoxelEmotions {
    fn default() -> Self {
        Self {
            base_emotions: [0.5; 8],
            emotion_vector: [0.0; 256],
            kaif: 0.0,
            prev_entropy: 0.0,
        }
    }
}

impl VoxelEmotions {
    pub fn update(&mut self, thoughts: &VoxelThoughts, dt: f32) {
        // Обновляем эмоциональный вектор из мыслей
        for i in 0..128 {
            self.emotion_vector[i] = 0.8 * self.emotion_vector[i] + 0.2 * thoughts.attention_focus[i];
        }
        for i in 0..128 {
            self.emotion_vector[128 + i] = 0.8 * self.emotion_vector[128 + i] + 0.2 * thoughts.working_memory[i];
        }
        
        // Обновляем базовые эмоции
        for i in 0..8 {
            let section_start = i * 32;
            let section_end = (section_start + 32).min(256);
            let mut sum = 0.0f32;
            for j in section_start..section_end {
                sum += self.emotion_vector[j].abs();
            }
            let avg = sum / 32.0;
            self.base_emotions[i] = 0.95 * self.base_emotions[i] + 0.05 * avg;
        }
        
        // Вычисляем кайф |dS/dt|
        self.compute_kaif(dt);
    }
    
    fn compute_kaif(&mut self, dt: f32) {
        // Энтропия эмоционального вектора
        let sum: f32 = self.emotion_vector.iter().map(|x| x.abs()).sum();
        if sum < 1e-8 {
            self.kaif = 0.0;
            return;
        }
        
        let mut entropy = 0.0f32;
        for &v in &self.emotion_vector {
            let p = v.abs() / sum;
            if p > 1e-10 {
                entropy -= p * p.log2();
            }
        }
        
        // Производная энтропии
        if dt > 0.0 {
            let d_entropy = (entropy - self.prev_entropy) / dt;
            self.kaif = d_entropy.abs();
        }
        
        self.prev_entropy = entropy;
    }
    
    pub fn dominant_emotion(&self) -> (EmotionType, f32) {
        let mut max_idx = 0;
        let mut max_val = self.base_emotions[0];
        
        for i in 1..8 {
            if self.base_emotions[i] > max_val {
                max_val = self.base_emotions[i];
                max_idx = i;
            }
        }
        
        (EmotionType::all()[max_idx], max_val)
    }
}

/// Память вокселя
#[derive(Debug, Clone)]
pub struct VoxelMemory {
    pub long_term: [f32; 256],
    pub episodes: Vec<[f32; 64]>,
    pub write_count: u32,
}

impl Default for VoxelMemory {
    fn default() -> Self {
        Self {
            long_term: [0.0; 256],
            episodes: Vec::with_capacity(16),
            write_count: 0,
        }
    }
}

impl VoxelMemory {
    pub fn store(&mut self, experience: &[f32], importance: f32) {
        let mut episode = [0.0f32; 64];
        let len = experience.len().min(64);
        episode[..len].copy_from_slice(&experience[..len]);
        
        // Интегрируем в долговременную память
        let learning_rate = 0.1 * importance;
        let start_idx = (self.write_count as usize % 4) * 64;
        for i in 0..64 {
            let idx = start_idx + i;
            if idx < 256 {
                self.long_term[idx] = (1.0 - learning_rate) * self.long_term[idx] + learning_rate * episode[i];
            }
        }
        
        // Сохраняем эпизод
        if self.episodes.len() >= 16 {
            self.episodes.remove(0);
        }
        self.episodes.push(episode);
        self.write_count += 1;
    }
    
    pub fn recall(&self, query: &[f32]) -> Option<[f32; 64]> {
        if self.episodes.is_empty() {
            return None;
        }
        
        let mut query_arr = [0.0f32; 64];
        let len = query.len().min(64);
        query_arr[..len].copy_from_slice(&query[..len]);
        
        let mut best_idx = 0;
        let mut best_sim = f32::NEG_INFINITY;
        
        for (i, ep) in self.episodes.iter().enumerate() {
            let sim: f32 = query_arr.iter().zip(ep.iter()).map(|(a, b)| a * b).sum();
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }
        
        Some(self.episodes[best_idx])
    }
}

/// Воксель - 9 КБ микро-организм
#[derive(Debug, Clone)]
pub struct Voxel {
    pub metadata: VoxelMetadata,
    pub sensors: VoxelSensors,
    pub physics: VoxelPhysics,
    pub thoughts: VoxelThoughts,
    pub emotions: VoxelEmotions,
    pub memory: VoxelMemory,
}

impl Voxel {
    pub fn new(id: u64) -> Self {
        Self {
            metadata: VoxelMetadata {
                id,
                ..Default::default()
            },
            sensors: VoxelSensors::default(),
            physics: VoxelPhysics::default(),
            thoughts: VoxelThoughts::default(),
            emotions: VoxelEmotions::default(),
            memory: VoxelMemory::default(),
        }
    }
    
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.metadata.position = pos;
        self
    }
    
    /// Главный цикл обновления
    pub fn update(&mut self, dt: f32) {
        self.metadata.age_ticks += 1;
        
        // 1. Физика
        let acceleration = self.physics.integrate(dt, self.metadata.mass);
        for i in 0..3 {
            self.metadata.velocity[i] += acceleration[i] * dt;
            self.metadata.position[i] += self.metadata.velocity[i] * dt;
        }
        
        // 2. Мысли
        let sensory_input = self.sensors.combined();
        self.thoughts.process(&sensory_input, dt);
        
        // 3. Эмоции
        self.emotions.update(&self.thoughts, dt);
        
        // 4. Память (сохраняем важный опыт)
        if self.emotions.kaif > 0.5 {
            let mut experience = Vec::with_capacity(72);
            experience.extend_from_slice(&self.thoughts.attention_focus[..32]);
            experience.extend_from_slice(&self.emotions.base_emotions);
            // Pad to 64
            experience.resize(64, 0.0);
            self.memory.store(&experience, self.emotions.kaif);
        }
        
        // 5. Жизненные показатели
        self.update_vitals(dt);
    }
    
    fn update_vitals(&mut self, dt: f32) {
        // Энергия тратится
        self.metadata.energy -= 0.001 * dt;
        
        // Высокий кайф восстанавливает энергию
        if self.emotions.kaif > 0.7 {
            self.metadata.energy += 0.002 * dt * self.emotions.kaif;
        }
        
        self.metadata.energy = self.metadata.energy.clamp(0.0, 1.0);
        
        // Здоровье
        if self.metadata.energy < 0.1 {
            self.metadata.health -= 0.001 * dt;
        } else {
            self.metadata.health += 0.0001 * dt;
        }
        
        self.metadata.health = self.metadata.health.clamp(0.0, 1.0);
    }
    
    pub fn receive_stimulus(&mut self, visual: Option<&[[f32; 3]; 32]>) {
        if let Some(v) = visual {
            self.sensors.visual = *v;
        }
    }
    
    pub fn get_kaif(&self) -> f32 {
        self.emotions.kaif
    }
    
    pub fn is_alive(&self) -> bool {
        self.metadata.health > 0.0
    }
}

/// Мир вокселей
pub struct VoxelWorld {
    pub voxels: HashMap<u64, Voxel>,
    pub max_voxels: usize,
    next_id: u64,
    pub current_tick: u64,
    
    // Статистика
    pub total_kaif: f32,
    pub avg_health: f32,
    pub avg_energy: f32,
}

impl VoxelWorld {
    pub fn new(max_voxels: usize) -> Self {
        Self {
            voxels: HashMap::with_capacity(max_voxels),
            max_voxels,
            next_id: 0,
            current_tick: 0,
            total_kaif: 0.0,
            avg_health: 1.0,
            avg_energy: 1.0,
        }
    }
    
    pub fn spawn(&mut self, position: [f32; 3]) -> u64 {
        if self.voxels.len() >= self.max_voxels {
            // Удаляем самый слабый
            if let Some((&id, _)) = self.voxels.iter().min_by(|a, b| {
                a.1.metadata.health.partial_cmp(&b.1.metadata.health).unwrap()
            }) {
                self.voxels.remove(&id);
            }
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        let voxel = Voxel::new(id).with_position(position);
        self.voxels.insert(id, voxel);
        
        id
    }
    
    pub fn update(&mut self, dt: f32) {
        self.current_tick += 1;
        
        // Собираем ID для удаления мёртвых
        let dead_ids: Vec<u64> = self.voxels
            .iter()
            .filter(|(_, v)| !v.is_alive())
            .map(|(&id, _)| id)
            .collect();
        
        for id in dead_ids {
            self.voxels.remove(&id);
        }
        
        // Обновляем живых
        // Используем параллельную обработку через iter_mut
        let voxels_vec: Vec<&mut Voxel> = self.voxels.values_mut().collect();
        
        // Собираем статистику
        let mut total_kaif = 0.0f32;
        let mut total_health = 0.0f32;
        let mut total_energy = 0.0f32;
        
        for voxel in voxels_vec {
            voxel.update(dt);
            total_kaif += voxel.emotions.kaif;
            total_health += voxel.metadata.health;
            total_energy += voxel.metadata.energy;
        }
        
        let n = self.voxels.len() as f32;
        if n > 0.0 {
            self.total_kaif = total_kaif;
            self.avg_health = total_health / n;
            self.avg_energy = total_energy / n;
        }
    }
    
    pub fn count(&self) -> usize {
        self.voxels.len()
    }
    
    pub fn get_emotion_distribution(&self) -> [f32; 8] {
        let mut dist = [0.0f32; 8];
        let n = self.voxels.len() as f32;
        
        if n < 1.0 {
            return [0.5; 8];
        }
        
        for voxel in self.voxels.values() {
            for i in 0..8 {
                dist[i] += voxel.emotions.base_emotions[i];
            }
        }
        
        for d in &mut dist {
            *d /= n;
        }
        
        dist
    }
}
