//! # Nucleotide - Базовая ячейка памяти (256 байт)
//!
//! Нуклеотид - фундаментальная единица хранения информации.
//! 
//! ## Структура (256 байт):
//! - 1 байт: base (A, T, G, C)
//! - 7 байт: epigenetic_tags
//! - 4 байта: quantum_noise (f32)
//! - 16 байт: histone_state
//! - 228 байт: semantic_vector (57 x f32)

use rand::Rng;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Тип нуклеотида (аналог ДНК)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NucleotideBase {
    Adenine = b'A',   // Память
    Thymine = b'T',   // Время
    Guanine = b'G',   // Генерация
    Cytosine = b'C',  // Связи
}

impl NucleotideBase {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..4) {
            0 => Self::Adenine,
            1 => Self::Thymine,
            2 => Self::Guanine,
            _ => Self::Cytosine,
        }
    }
    
    pub fn as_char(&self) -> char {
        *self as u8 as char
    }
}

/// Эпигенетические модификации
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EpigeneticTag {
    Methylation = b'M',     // Подавление
    Acetylation = b'A',     // Активация
    Phosphorylation = b'P', // Сигнализация
    Ubiquitination = b'U',  // Деградация
}

/// Состояние гистонов
#[derive(Debug, Clone, Copy)]
pub struct HistoneState {
    pub compaction: f32,      // Степень компактизации [0-1]
    pub accessibility: f32,   // Доступность для чтения [0-1]
    pub stability: f32,       // Стабильность [0-1]
    pub modification_count: u32,
}

impl Default for HistoneState {
    fn default() -> Self {
        Self {
            compaction: 0.5,
            accessibility: 0.5,
            stability: 0.8,
            modification_count: 0,
        }
    }
}

/// Размер семантического вектора
pub const SEMANTIC_VECTOR_SIZE: usize = 57; // 57 * 4 = 228 байт

/// Нуклеотид - 256 байт
#[derive(Debug, Clone)]
pub struct Nucleotide {
    pub base: NucleotideBase,
    pub epigenetic_tags: [(EpigeneticTag, f32); 4], // До 4 меток
    pub epigenetic_count: u8,
    pub quantum_noise: f32,
    pub histone_state: HistoneState,
    pub semantic_vector: [f32; SEMANTIC_VECTOR_SIZE],
    
    // Метаданные
    pub energy: f32,
    pub creation_tick: u64,
    pub last_access_tick: u64,
    pub access_count: u32,
}

impl Default for Nucleotide {
    fn default() -> Self {
        Self {
            base: NucleotideBase::Adenine,
            epigenetic_tags: [(EpigeneticTag::Methylation, 0.0); 4],
            epigenetic_count: 0,
            quantum_noise: 0.0,
            histone_state: HistoneState::default(),
            semantic_vector: [0.0; SEMANTIC_VECTOR_SIZE],
            energy: 1.0,
            creation_tick: 0,
            last_access_tick: 0,
            access_count: 0,
        }
    }
}

impl Nucleotide {
    /// Создание нового нуклеотида
    pub fn new(base: NucleotideBase) -> Self {
        Self {
            base,
            ..Default::default()
        }
    }
    
    /// Создание случайного нуклеотида
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.1).unwrap();
        
        let mut semantic_vector = [0.0f32; SEMANTIC_VECTOR_SIZE];
        for v in &mut semantic_vector {
            *v = normal.sample(&mut rng) as f32;
        }
        
        Self {
            base: NucleotideBase::random(),
            quantum_noise: rng.gen_range(-1.0..1.0),
            semantic_vector,
            ..Default::default()
        }
    }
    
    /// Обновление нуклеотида на один тик
    pub fn update(&mut self, dt: f32, current_tick: u64) {
        let mut rng = rand::thread_rng();
        
        // Обновляем квантовый шум
        self.quantum_noise = rng.gen_range(-0.1..0.1) * self.histone_state.accessibility;
        
        // Затухание энергии
        self.energy = (self.energy * (1.0 - 0.001 * dt)).max(0.1);
        
        // Обновление эпигенетических меток
        self.update_epigenetic_tags(dt);
        
        // Обновление гистонов
        self.update_histone_state(dt);
        
        self.last_access_tick = current_tick;
    }
    
    fn update_epigenetic_tags(&mut self, dt: f32) {
        let mut rng = rand::thread_rng();
        
        // Метки затухают со временем
        for i in 0..self.epigenetic_count as usize {
            self.epigenetic_tags[i].1 *= 1.0 - 0.01 * dt;
            if self.epigenetic_tags[i].1 < 0.01 {
                // Удаляем метку (сдвигаем остальные)
                for j in i..3 {
                    self.epigenetic_tags[j] = self.epigenetic_tags[j + 1];
                }
                self.epigenetic_count = self.epigenetic_count.saturating_sub(1);
            }
        }
        
        // Случайные новые модификации
        if rng.gen::<f32>() < 0.001 * dt && self.epigenetic_count < 4 {
            let new_tag = match rng.gen_range(0..4) {
                0 => EpigeneticTag::Methylation,
                1 => EpigeneticTag::Acetylation,
                2 => EpigeneticTag::Phosphorylation,
                _ => EpigeneticTag::Ubiquitination,
            };
            self.epigenetic_tags[self.epigenetic_count as usize] = (new_tag, rng.gen_range(0.3..1.0));
            self.epigenetic_count += 1;
        }
    }
    
    fn update_histone_state(&mut self, dt: f32) {
        // Ищем метилирование и ацетилирование
        let mut methylation = 0.0f32;
        let mut acetylation = 0.0f32;
        
        for i in 0..self.epigenetic_count as usize {
            match self.epigenetic_tags[i].0 {
                EpigeneticTag::Methylation => methylation = self.epigenetic_tags[i].1,
                EpigeneticTag::Acetylation => acetylation = self.epigenetic_tags[i].1,
                _ => {}
            }
        }
        
        // Компактизация
        let target_compaction = 0.5 + 0.3 * methylation - 0.3 * acetylation;
        self.histone_state.compaction += (target_compaction - self.histone_state.compaction) * 0.1 * dt;
        
        // Доступность обратна компактизации
        self.histone_state.accessibility = 1.0 - self.histone_state.compaction * 0.8;
        
        // Стабильность увеличивается с возрастом
        self.histone_state.stability = (self.histone_state.stability + 0.0001 * dt).min(1.0);
    }
    
    /// Интеграция опыта в семантический вектор
    pub fn integrate_experience(&mut self, experience: &[f32], dt: f32) {
        let learning_rate = 0.01 * self.histone_state.accessibility * self.energy * dt;
        
        let len = experience.len().min(SEMANTIC_VECTOR_SIZE);
        for i in 0..len {
            let gradient = experience[i] - self.semantic_vector[i];
            self.semantic_vector[i] += learning_rate * gradient;
        }
        
        // Нормализация
        let norm: f32 = self.semantic_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 10.0 {
            for v in &mut self.semantic_vector {
                *v /= norm / 10.0;
            }
        }
    }
    
    /// Добавление эпигенетической метки
    pub fn add_epigenetic_tag(&mut self, tag: EpigeneticTag, strength: f32) {
        if self.epigenetic_count < 4 {
            self.epigenetic_tags[self.epigenetic_count as usize] = (tag, strength.min(1.0));
            self.epigenetic_count += 1;
            self.histone_state.modification_count += 1;
        }
    }
    
    /// Вычисление сходства с другим нуклеотидом
    pub fn similarity(&self, other: &Nucleotide) -> f32 {
        let mut dot = 0.0f32;
        let mut norm1 = 0.0f32;
        let mut norm2 = 0.0f32;
        
        for i in 0..SEMANTIC_VECTOR_SIZE {
            dot += self.semantic_vector[i] * other.semantic_vector[i];
            norm1 += self.semantic_vector[i] * self.semantic_vector[i];
            norm2 += other.semantic_vector[i] * other.semantic_vector[i];
        }
        
        let norm = (norm1 * norm2).sqrt();
        if norm < 1e-6 {
            0.0
        } else {
            dot / norm
        }
    }
    
    /// Сериализация в байты (256 байт)
    pub fn to_bytes(&self) -> [u8; 256] {
        let mut data = [0u8; 256];
        
        // Байт 0: base
        data[0] = self.base as u8;
        
        // Байты 1-8: epigenetic tags
        for i in 0..self.epigenetic_count as usize {
            let (tag, strength) = self.epigenetic_tags[i];
            data[1 + i * 2] = tag as u8;
            data[2 + i * 2] = (strength * 255.0) as u8;
        }
        
        // Байты 9-12: quantum_noise
        data[9..13].copy_from_slice(&self.quantum_noise.to_le_bytes());
        
        // Байты 13-28: histone_state (4 x f32)
        data[13..17].copy_from_slice(&self.histone_state.compaction.to_le_bytes());
        data[17..21].copy_from_slice(&self.histone_state.accessibility.to_le_bytes());
        data[21..25].copy_from_slice(&self.histone_state.stability.to_le_bytes());
        data[25..29].copy_from_slice(&(self.histone_state.modification_count as f32).to_le_bytes());
        
        // Байты 29-256: semantic_vector (57 x f32 = 228 байт)
        for (i, &v) in self.semantic_vector.iter().enumerate() {
            let offset = 29 + i * 4;
            if offset + 4 <= 256 {
                data[offset..offset + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        
        data
    }
}

/// Пул нуклеотидов для параллельной обработки
pub struct NucleotidePool {
    pub nucleotides: Vec<Nucleotide>,
    pub size: usize,
    pub current_tick: AtomicU64,
    pub total_updates: AtomicU64,
}

impl NucleotidePool {
    /// Создание пула
    pub fn new(size: usize) -> Self {
        Self {
            nucleotides: Vec::with_capacity(size),
            size,
            current_tick: AtomicU64::new(0),
            total_updates: AtomicU64::new(0),
        }
    }
    
    /// Инициализация с случайными нуклеотидами
    pub fn initialize(&mut self) {
        println!("🧬 Инициализация пула из {} нуклеотидов...", self.size);
        
        // Параллельная инициализация с Rayon!
        self.nucleotides = (0..self.size)
            .into_par_iter()
            .map(|_| Nucleotide::random())
            .collect();
        
        println!("✅ Пул инициализирован!");
    }
    
    /// Параллельное обновление всех нуклеотидов
    pub fn update_all(&mut self, dt: f32) {
        let tick = self.current_tick.fetch_add(1, Ordering::Relaxed);
        
        // Параллельное обновление с Rayon - использует все 36 потоков!
        self.nucleotides.par_iter_mut().for_each(|nuc| {
            nuc.update(dt, tick);
        });
        
        self.total_updates.fetch_add(self.size as u64, Ordering::Relaxed);
    }
    
    /// Интеграция опыта во все нуклеотиды
    pub fn integrate_experience_all(&mut self, experience: &[f32], dt: f32) {
        self.nucleotides.par_iter_mut().for_each(|nuc| {
            nuc.integrate_experience(experience, dt);
        });
    }
    
    /// Поиск похожих нуклеотидов
    pub fn find_similar(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        let query_nuc = {
            let mut n = Nucleotide::default();
            let len = query.len().min(SEMANTIC_VECTOR_SIZE);
            n.semantic_vector[..len].copy_from_slice(&query[..len]);
            n
        };
        
        // Параллельный поиск
        let mut similarities: Vec<(usize, f32)> = self.nucleotides
            .par_iter()
            .enumerate()
            .map(|(i, nuc)| (i, nuc.similarity(&query_nuc)))
            .collect();
        
        // Сортируем по убыванию сходства
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        similarities.truncate(top_k);
        similarities
    }
    
    /// Получение статистики
    pub fn get_statistics(&self) -> NucleotidePoolStats {
        let total_energy: f32 = self.nucleotides.par_iter().map(|n| n.energy).sum();
        let total_noise: f32 = self.nucleotides.par_iter().map(|n| n.quantum_noise.abs()).sum();
        
        NucleotidePoolStats {
            size: self.size,
            current_tick: self.current_tick.load(Ordering::Relaxed),
            total_updates: self.total_updates.load(Ordering::Relaxed),
            mean_energy: total_energy / self.size as f32,
            mean_quantum_noise: total_noise / self.size as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NucleotidePoolStats {
    pub size: usize,
    pub current_tick: u64,
    pub total_updates: u64,
    pub mean_energy: f32,
    pub mean_quantum_noise: f32,
}
