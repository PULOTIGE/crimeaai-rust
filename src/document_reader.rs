use std::fs;
use std::path::Path;

/// Читалка документов с поддержкой PDF и DJVU
pub struct DocumentReader {
    pub supported_formats: Vec<String>,
}

impl DocumentReader {
    pub fn new() -> Self {
        Self {
            supported_formats: vec![
                // Текстовые
                "txt".to_string(), "md".to_string(), "json".to_string(), 
                "csv".to_string(), "log".to_string(), "xml".to_string(),
                // Код
                "rs".to_string(), "py".to_string(), "js".to_string(), 
                "html".to_string(), "css".to_string(), "java".to_string(),
                "cpp".to_string(), "c".to_string(), "h".to_string(),
                // Документы
                "pdf".to_string(),
                // DJVU пока заглушка (требует внешние библиотеки)
                "djvu".to_string(), "djv".to_string(),
            ],
        }
    }
    
    /// Проверка поддержки формата
    pub fn is_supported(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.supported_formats.contains(&ext_str.to_lowercase());
            }
        }
        false
    }
    
    /// Чтение файла с автоопределением формата
    pub fn read_file(&self, path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("Файл не найден: {:?}", path));
        }
        
        if !self.is_supported(path) {
            return Err(format!("Неподдерживаемый формат: {:?}", path.extension()));
        }
        
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        match ext.as_str() {
            "pdf" => self.read_pdf(path),
            "djvu" | "djv" => self.read_djvu(path),
            _ => self.read_text(path),
        }
    }
    
    /// Чтение текстового файла
    fn read_text(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Ошибка чтения текстового файла: {}", e))
    }
    
    /// Чтение PDF файла
    fn read_pdf(&self, path: &Path) -> Result<String, String> {
        // Используем простое извлечение из PDF bytes
        match fs::read(path) {
            Ok(bytes) => {
                let text = Self::extract_text_from_pdf_bytes(&bytes);
                if text.is_empty() {
                    // Если не удалось извлечь текст, возвращаем информацию
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
        // Ищем текстовые фрагменты в PDF
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
    
    /// Чтение DJVU файла (заглушка)
    fn read_djvu(&self, path: &Path) -> Result<String, String> {
        // DJVU требует внешних библиотек (djvulibre)
        // Пока возвращаем заглушку
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
    
    /// Извлечение обучающих данных из текста
    pub fn extract_training_data(&self, content: &str) -> Vec<String> {
        let mut examples = Vec::new();
        
        // Разбивка по абзацам
        for paragraph in content.split("\n\n") {
            let trimmed = paragraph.trim();
            if !trimmed.is_empty() && trimmed.len() > 15 {
                examples.push(trimmed.to_string());
            }
        }
        
        // Если абзацев мало, разбиваем по предложениям
        if examples.len() < 3 {
            examples.clear();
            let sentences: Vec<&str> = content
                .split(&['.', '!', '?', '\n'][..])
                .collect();
            
            for sentence in sentences {
                let trimmed = sentence.trim();
                if !trimmed.is_empty() && trimmed.len() > 15 {
                    examples.push(trimmed.to_string());
                }
            }
        }
        
        examples
    }
    
    /// Статистика файла
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
    
    /// Валидация данных
    pub fn validate_training_data(&self, data: &[String]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Нет данных для обучения".to_string());
        }
        
        if data.len() < 3 {
            return Err(format!("Слишком мало примеров: {} (минимум 3)", data.len()));
        }
        
        let avg_length: usize = data.iter().map(|s| s.len()).sum::<usize>() / data.len();
        if avg_length < 15 {
            return Err("Примеры слишком короткие (минимум 15 символов)".to_string());
        }
        
        Ok(())
    }
}

impl Default for DocumentReader {
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
            "📄 Строк: {}\n💬 Слов: {}\n🔤 Символов: {}\n📦 Байт: {}",
            self.lines, self.words, self.chars, self.bytes
        )
    }
}
