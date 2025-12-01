//! # Ecosystem - Объединение всех компонентов

use crate::{
    NucleotidePool, VoxelWorld, PatternDatabase, KaifEngine, ConceptSearcher,
};
use std::time::Instant;

/// Главная экосистема
pub struct Ecosystem {
    pub nucleotides: NucleotidePool,
    pub voxels: VoxelWorld,
    pub patterns: PatternDatabase,
    pub kaif: KaifEngine,
    pub concepts: ConceptSearcher,
    
    pub running: bool,
    pub paused: bool,
    pub start_time: Instant,
    pub total_ticks: u64,
    pub fps: f32,
    
    last_frame_time: Instant,
    fps_samples: Vec<f32>,
}

impl Ecosystem {
    pub fn new(nucleotide_count: usize, max_voxels: usize, max_patterns: usize) -> Self {
        println!("🚀 Создание CrimeaAI Ecosystem...");
        
        let mut eco = Self {
            nucleotides: NucleotidePool::new(nucleotide_count),
            voxels: VoxelWorld::new(max_voxels),
            patterns: PatternDatabase::new(max_patterns),
            kaif: KaifEngine::new(),
            concepts: ConceptSearcher::default(),
            
            running: false,
            paused: false,
            start_time: Instant::now(),
            total_ticks: 0,
            fps: 0.0,
            
            last_frame_time: Instant::now(),
            fps_samples: Vec::with_capacity(60),
        };
        
        // Инициализация
        eco.nucleotides.initialize();
        eco.patterns.generate_random(100);
        
        // Регистрируем компоненты в KaifEngine
        eco.kaif.register_component("nucleotides", vec![0.0; 64], 0.3);
        eco.kaif.register_component("voxels", vec![0.0; 64], 0.5);
        eco.kaif.register_component("emotions", vec![0.0; 64], 0.2);
        
        // Спавним начальные воксели
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let pos = [
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
                0.0,
            ];
            eco.voxels.spawn(pos);
        }
        
        println!("✅ Экосистема создана!");
        println!("   🧬 Нуклеотидов: {}", nucleotide_count);
        println!("   🌍 Вокселей: {}", eco.voxels.count());
        println!("   💡 Паттернов: {}", eco.patterns.count());
        
        eco
    }
    
    /// Обновление экосистемы
    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        
        self.total_ticks += 1;
        
        // Обновляем нуклеотиды (параллельно!)
        self.nucleotides.update_all(dt);
        
        // Обновляем воксели
        self.voxels.update(dt);
        
        // Обновляем KaifEngine
        // Собираем данные из нуклеотидов
        if self.nucleotides.nucleotides.len() > 0 {
            let sample: Vec<f32> = self.nucleotides.nucleotides
                .iter()
                .take(64)
                .flat_map(|n| n.semantic_vector.iter().take(1).cloned())
                .collect();
            self.kaif.update_component("nucleotides", sample);
        }
        
        // Собираем эмоции из вокселей
        let emotions: Vec<f32> = self.voxels.voxels
            .values()
            .take(8)
            .flat_map(|v| v.emotions.base_emotions.iter().cloned())
            .collect();
        if !emotions.is_empty() {
            self.kaif.update_component("emotions", emotions);
        }
        
        self.kaif.update(dt);
        
        // FPS
        let now = Instant::now();
        let frame_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        if frame_time > 0.0 {
            self.fps_samples.push(1.0 / frame_time);
            if self.fps_samples.len() > 60 {
                self.fps_samples.remove(0);
            }
            self.fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
        }
    }
    
    /// Поиск концептов
    pub fn search_concepts(&mut self) {
        let concepts = self.concepts.search_simulated();
        println!("🔍 Найдено {} концептов", concepts.len());
    }
    
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            println!("⏸️ Пауза");
        } else {
            println!("▶️ Продолжение");
        }
    }
    
    pub fn get_stats(&self) -> EcosystemStats {
        EcosystemStats {
            ticks: self.total_ticks,
            fps: self.fps,
            kaif: self.kaif.get_kaif(),
            kaif_state: self.kaif.get_state().as_str().to_string(),
            nucleotide_count: self.nucleotides.size,
            voxel_count: self.voxels.count(),
            avg_health: self.voxels.avg_health,
            avg_energy: self.voxels.avg_energy,
            concept_count: self.concepts.count(),
            uptime_secs: self.start_time.elapsed().as_secs_f32(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EcosystemStats {
    pub ticks: u64,
    pub fps: f32,
    pub kaif: f32,
    pub kaif_state: String,
    pub nucleotide_count: usize,
    pub voxel_count: usize,
    pub avg_health: f32,
    pub avg_energy: f32,
    pub concept_count: usize,
    pub uptime_secs: f32,
}
