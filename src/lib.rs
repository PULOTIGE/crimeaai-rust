// AI Chat Application with Document Processing Library

pub mod ai_model;
pub mod file_processor;
pub mod document_reader;
pub mod chat_ui;

// Re-export main types
pub use ai_model::AIModel;
pub use file_processor::{FileProcessor, FileStats};
pub use document_reader::DocumentReader;
pub use chat_ui::{ChatUI, ChatMessage, AppMode, TrainingStatus};
