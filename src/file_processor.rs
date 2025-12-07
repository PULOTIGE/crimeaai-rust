use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;

/// Обработчик файлов для загрузки обучающих данных
pub struct FileProcessor {
    pub supported_extensions: Vec<String>,
}

impl FileProcessor {
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "json".to_string(),
                "csv".to_string(),
                "log".to_string(),
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "html".to_string(),
                "xml".to_string(),
            ],
        }
    }
    
    /// Проверка поддерживаемого формата
    pub fn is_supported(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.supported_extensions.contains(&ext_str.to_lowercase());
            }
        }
        false
    }
    
    /// Чтение файла
    pub fn read_file(&self, path: &Path) -> Result<String, String> {
        if !self.is_supported(path) {
            return Err(format!("Неподдерживаемый формат файла: {:?}", path.extension()));
        }
        
        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => Err(format!("Ошибка чтения файла: {}", e)),
        }
    }
    
    /// Чтение всех файлов из директории
    pub fn read_directory(&self, dir_path: &Path) -> Result<Vec<(PathBuf, String)>, String> {
        let mut files_content = Vec::new();
        
        if !dir_path.is_dir() {
            return Err("Указанный путь не является директорией".to_string());
        }
        
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Ошибка чтения директории: {}", e)),
        };
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && self.is_supported(&path) {
                    match self.read_file(&path) {
                        Ok(content) => files_content.push((path, content)),
                        Err(e) => eprintln!("Пропуск файла {:?}: {}", path, e),
                    }
                }
            }
        }
        
        Ok(files_content)
    }
    
    /// Извлечение обучающих примеров из текста
    pub fn extract_training_data(&self, content: &str) -> Vec<String> {
        // Разбиваем на предложения/абзацы
        let mut examples = Vec::new();
        
        // Разбивка по абзацам
        for paragraph in content.split("\n\n") {
            let trimmed = paragraph.trim();
            if !trimmed.is_empty() && trimmed.len() > 10 {
                examples.push(trimmed.to_string());
            }
        }
        
        // Если абзацев мало, разбиваем по предложениям
        if examples.len() < 5 {
            examples.clear();
            for sentence in content.split(&['.', '!', '?', '\n'][..]) {
                let trimmed = sentence.trim();
                if !trimmed.is_empty() && trimmed.len() > 10 {
                    examples.push(trimmed.to_string());
                }
            }
        }
        
        examples
    }
    
    /// Получение статистики по файлу
    pub fn get_file_stats(&self, content: &str) -> FileStats {
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        let bytes = content.len();
        
        FileStats {
            lines,
            words,
            chars,
            bytes,
        }
    }
    
    /// Валидация данных для обучения
    pub fn validate_training_data(&self, data: &[String]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Нет данных для обучения".to_string());
        }
        
        if data.len() < 3 {
            return Err("Слишком мало примеров для обучения (минимум 3)".to_string());
        }
        
        let avg_length: usize = data.iter().map(|s| s.len()).sum::<usize>() / data.len();
        if avg_length < 10 {
            return Err("Примеры слишком короткие (минимум 10 символов)".to_string());
        }
        
        Ok(())
    }
}

impl Default for FileProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub bytes: usize,
}

impl FileStats {
    pub fn format(&self) -> String {
        format!(
            "Строк: {}\nСлов: {}\nСимволов: {}\nБайт: {}",
            self.lines, self.words, self.chars, self.bytes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_processor() {
        let processor = FileProcessor::new();
        assert!(processor.supported_extensions.contains(&"txt".to_string()));
    }
    
    #[test]
    fn test_extract_training_data() {
        let processor = FileProcessor::new();
        let content = "Первое предложение.\n\nВторое предложение.";
        let data = processor.extract_training_data(content);
        assert!(!data.is_empty());
    }
    
    #[test]
    fn test_file_stats() {
        let processor = FileProcessor::new();
        let content = "Hello world\nTest line";
        let stats = processor.get_file_stats(content);
        assert_eq!(stats.lines, 2);
        assert_eq!(stats.words, 4);
    }
}
