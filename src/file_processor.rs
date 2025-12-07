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
                // Текстовые
                "txt".to_string(),
                "md".to_string(),
                "json".to_string(),
                "csv".to_string(),
                "log".to_string(),
                "xml".to_string(),
                // Код
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "html".to_string(),
                "css".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                // Документы
                "pdf".to_string(),
                "djvu".to_string(),
                "djv".to_string(),
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
    
    /// Чтение файла с поддержкой PDF и DJVU
    pub fn read_file(&self, path: &Path) -> Result<String, String> {
        if !self.is_supported(path) {
            return Err(format!("Неподдерживаемый формат файла: {:?}", path.extension()));
        }
        
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        match ext.as_str() {
            "pdf" => self.read_pdf(path),
            "djvu" | "djv" => self.read_djvu(path),
            _ => {
                // Обычные текстовые файлы
                match fs::read_to_string(path) {
                    Ok(content) => Ok(content),
                    Err(e) => Err(format!("Ошибка чтения файла: {}", e)),
                }
            }
        }
    }
    
    /// Чтение PDF файла
    fn read_pdf(&self, path: &Path) -> Result<String, String> {
        match fs::read(path) {
            Ok(bytes) => {
                let text = Self::extract_text_from_pdf_bytes(&bytes);
                if text.is_empty() {
                    Ok(format!(
                        "📄 PDF файл загружен ({} байт)\n\n\
                         ⚠️ Автоматическое извлечение текста из PDF может быть неполным.\n\n\
                         💡 Для лучшего качества обучения:\n\
                         1. Конвертируйте PDF → TXT онлайн\n\
                         2. Или используйте текстовый редактор для копирования\n\
                         3. Сохраните как .txt файл и загрузите снова\n\n\
                         Файл: {:?}",
                        bytes.len(),
                        path.file_name().unwrap_or_default()
                    ))
                } else {
                    Ok(format!("📄 PDF текст (базовое извлечение):\n\n{}\n\n\
                               ℹ️ Извлечено методом поиска текстовых блоков", text))
                }
            }
            Err(e) => Err(format!("Ошибка чтения PDF файла: {}", e))
        }
    }
    
    /// Извлечение текста из PDF байтов
    fn extract_text_from_pdf_bytes(bytes: &[u8]) -> String {
        let text = String::from_utf8_lossy(bytes);
        let mut result = String::new();
        
        // Простой метод: ищем текст между BT и ET (text objects в PDF)
        for part in text.split("BT") {
            if let Some(end) = part.find("ET") {
                let text_part = &part[..end];
                // Убираем PDF команды и извлекаем читаемый текст
                for line in text_part.lines() {
                    if line.contains("Tj") || line.contains("TJ") {
                        // Извлекаем текст из команд Tj
                        if let Some(start) = line.find('(') {
                            if let Some(end) = line[start..].find(')') {
                                let extracted = &line[start+1..start+end];
                                result.push_str(extracted);
                                result.push(' ');
                            }
                        }
                    }
                }
            }
        }
        
        result.trim().to_string()
    }
    
    /// Чтение DJVU файла
    fn read_djvu(&self, path: &Path) -> Result<String, String> {
        Err(format!(
            "❌ DJVU пока не поддерживается напрямую\n\n\
             📝 Решение:\n\
             1. Конвертируйте DJVU → PDF онлайн:\n\
                • https://djvu2pdf.com/\n\
                • https://www.zamzar.com/convert/djvu-to-pdf/\n\n\
             2. Или DJVU → TXT:\n\
                • Используйте djvutxt утилиту\n\
                • Или OCR инструмент\n\n\
             Файл: {:?}", 
            path.file_name().unwrap_or_default()
        ))
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
            if !trimmed.is_empty() && trimmed.len() > 3 {  // Уменьшили с 10 до 3
                examples.push(trimmed.to_string());
            }
        }
        
        // Если абзацев мало, разбиваем по предложениям
        if examples.len() < 3 {  // Уменьшили с 5 до 3
            examples.clear();
            for sentence in content.split(&['.', '!', '?', '\n'][..]) {
                let trimmed = sentence.trim();
                if !trimmed.is_empty() && trimmed.len() > 3 {  // Уменьшили с 10 до 3
                    examples.push(trimmed.to_string());
                }
            }
        }
        
        // Если всё ещё мало, берём весь текст целиком
        if examples.is_empty() && !content.trim().is_empty() {
            examples.push(content.trim().to_string());
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
    
    /// Валидация данных для обучения (упрощённая)
    pub fn validate_training_data(&self, data: &[String]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Нет данных для обучения. Файл пустой или не содержит текста.".to_string());
        }
        
        // Убрали проверку минимума примеров - даже 1 пример это ок!
        
        // Проверяем, что хотя бы один пример имеет приличную длину
        let has_decent_example = data.iter().any(|s| s.len() > 5);
        if !has_decent_example {
            return Err(format!(
                "Все примеры слишком короткие.\n\
                 📊 Найдено примеров: {}\n\
                 💡 Добавьте больше текста в файл (минимум 5 символов на пример)",
                data.len()
            ));
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
