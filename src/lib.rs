// Hybrid AI Chat + Voxels Application Library

pub mod ai_model;
pub mod file_processor;
pub mod chat_ui;
pub mod voxel;
pub mod evolution;
pub mod system_monitor;

// Re-export main types
pub use ai_model::AIModel;
pub use file_processor::{FileProcessor, FileStats};
pub use chat_ui::{ChatUI, ChatMessage, AppMode, TrainingStatus};
pub use voxel::{Voxel, VoxelWorld, Genome};
pub use evolution::EvolutionEngine;
pub use system_monitor::SystemMonitor;
