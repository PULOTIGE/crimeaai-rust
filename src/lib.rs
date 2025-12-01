//! # CrimeaAI Ecosystem
//! 
//! 🧠 AI-экосистема с биологическими структурами данных
//! 
//! ## Компоненты:
//! - `nucleotide` - Нуклеотид (256 байт) - базовая ячейка памяти
//! - `voxel` - Воксель (9 КБ) - микро-организм
//! - `light_pattern` - Паттерн освещения (1 КБ)
//! - `kaif` - Движок кайфа (производная энтропии)
//! - `concept` - Поиск концептов

pub mod nucleotide;
pub mod voxel;
pub mod light_pattern;
pub mod kaif;
pub mod concept;
pub mod scheduler;
pub mod world;

pub use nucleotide::{Nucleotide, NucleotideBase, NucleotidePool};
pub use voxel::{Voxel, VoxelWorld};
pub use light_pattern::{LightPattern, PatternDatabase};
pub use kaif::{KaifEngine, KaifState};
pub use concept::{Concept, ConceptSearcher};
pub use world::Ecosystem;
