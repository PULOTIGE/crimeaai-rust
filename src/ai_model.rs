use nalgebra::{DMatrix, DVector};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Простая нейронная сеть с поддержкой fp64 для высокоточного обучения
#[derive(Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub layers: Vec<Layer>,
    pub learning_rate: f64,
    pub vocab: HashMap<String, usize>,
    pub reverse_vocab: HashMap<usize, String>,
    pub embedding_dim: usize,
    pub hidden_dim: usize,
    pub context_length: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
    pub activation: ActivationType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Tanh,
    Sigmoid,
    Softmax,
}

impl AIModel {
    pub fn new(embedding_dim: usize, hidden_dim: usize, context_length: usize) -> Self {
        let mut model = Self {
            layers: Vec::new(),
            learning_rate: 0.001,
            vocab: HashMap::new(),
            reverse_vocab: HashMap::new(),
            embedding_dim,
            hidden_dim,
            context_length,
        };
        
        // Инициализация базового словаря
        model.init_vocab();
        
        // Создание слоев нейронной сети
        model.init_layers();
        
        model
    }
    
    fn init_vocab(&mut self) {
        // Базовый русский и английский словарь
        let base_words = vec![
            "привет", "hello", "как", "дела", "что", "это", "я", "ты", "он", "она",
            "мы", "вы", "они", "и", "в", "на", "с", "по", "для", "от", "к", "о",
            "the", "a", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "should",
            "<PAD>", "<START>", "<END>", "<UNK>",
        ];
        
        for (idx, word) in base_words.iter().enumerate() {
            self.vocab.insert(word.to_string(), idx);
            self.reverse_vocab.insert(idx, word.to_string());
        }
    }
    
    fn init_layers(&mut self) {
        let mut rng = rand::thread_rng();
        let vocab_size = self.vocab.len();
        
        // Embedding layer
        let embedding_layer = Layer {
            weights: (0..vocab_size)
                .map(|_| (0..self.embedding_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.embedding_dim],
            activation: ActivationType::ReLU,
        };
        
        // Hidden layer 1
        let hidden1 = Layer {
            weights: (0..self.embedding_dim * self.context_length)
                .map(|_| (0..self.hidden_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.hidden_dim],
            activation: ActivationType::Tanh,
        };
        
        // Hidden layer 2
        let hidden2 = Layer {
            weights: (0..self.hidden_dim)
                .map(|_| (0..self.hidden_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.hidden_dim],
            activation: ActivationType::Tanh,
        };
        
        // Output layer
        let output_layer = Layer {
            weights: (0..self.hidden_dim)
                .map(|_| (0..vocab_size)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; vocab_size],
            activation: ActivationType::Softmax,
        };
        
        self.layers.push(embedding_layer);
        self.layers.push(hidden1);
        self.layers.push(hidden2);
        self.layers.push(output_layer);
    }
    
    /// Прямое распространение
    pub fn forward(&self, input_tokens: &[usize]) -> Vec<f64> {
        let mut activations = Vec::new();
        
        // Embedding
        for &token in input_tokens.iter().take(self.context_length) {
            if token < self.layers[0].weights.len() {
                activations.extend_from_slice(&self.layers[0].weights[token]);
            } else {
                activations.extend(vec![0.0; self.embedding_dim]);
            }
        }
        
        // Дополняем до нужной длины
        while activations.len() < self.embedding_dim * self.context_length {
            activations.push(0.0);
        }
        
        // Проход через скрытые слои
        for layer in self.layers.iter().skip(1) {
            activations = self.apply_layer(&activations, layer);
        }
        
        activations
    }
    
    fn apply_layer(&self, input: &[f64], layer: &Layer) -> Vec<f64> {
        let output_size = layer.biases.len();
        let input_size = if layer.weights.is_empty() { 0 } else { layer.weights[0].len() };
        
        let mut output = vec![0.0; output_size];
        
        for i in 0..output_size {
            let mut sum = layer.biases[i];
            for j in 0..input.len().min(layer.weights.len()) {
                if i < layer.weights[j].len() {
                    sum += input[j] * layer.weights[j][i];
                }
            }
            output[i] = sum;
        }
        
        // Применение функции активации
        match layer.activation {
            ActivationType::ReLU => output.iter().map(|&x| x.max(0.0)).collect(),
            ActivationType::Tanh => output.iter().map(|&x| x.tanh()).collect(),
            ActivationType::Sigmoid => output.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect(),
            ActivationType::Softmax => {
                let max_val = output.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exp_vals: Vec<f64> = output.iter().map(|&x| (x - max_val).exp()).collect();
                let sum: f64 = exp_vals.iter().sum();
                exp_vals.iter().map(|&x| x / sum).collect()
            }
        }
    }
    
    /// Генерация ответа
    pub fn generate(&self, input_text: &str, max_length: usize) -> String {
        let tokens = self.tokenize(input_text);
        let mut generated_tokens = tokens.clone();
        
        for _ in 0..max_length {
            let context: Vec<usize> = generated_tokens
                .iter()
                .rev()
                .take(self.context_length)
                .rev()
                .cloned()
                .collect();
            
            let probs = self.forward(&context);
            let next_token = self.sample_token(&probs);
            
            // Проверка на конец генерации
            if let Some(token_str) = self.reverse_vocab.get(&next_token) {
                if token_str == "<END>" {
                    break;
                }
            }
            
            generated_tokens.push(next_token);
        }
        
        self.decode(&generated_tokens[tokens.len()..])
    }
    
    /// Обучение на данных
    pub fn train(&mut self, texts: &[String], epochs: usize, progress_callback: impl Fn(usize, usize, f64)) {
        for epoch in 0..epochs {
            let mut total_loss = 0.0;
            let mut num_samples = 0;
            
            for text in texts {
                let tokens = self.tokenize(text);
                
                // Создаем обучающие пары (контекст -> следующее слово)
                for i in 0..(tokens.len().saturating_sub(1)) {
                    let context_end = (i + 1).min(tokens.len());
                    let context_start = context_end.saturating_sub(self.context_length);
                    let context = &tokens[context_start..context_end];
                    let target = tokens[context_end.min(tokens.len() - 1)];
                    
                    // Forward pass
                    let output = self.forward(context);
                    
                    // Вычисление loss
                    let loss = self.compute_loss(&output, target);
                    total_loss += loss;
                    num_samples += 1;
                    
                    // Backward pass (упрощенный градиентный спуск)
                    self.update_weights(context, target, &output);
                }
            }
            
            let avg_loss = if num_samples > 0 { total_loss / num_samples as f64 } else { 0.0 };
            progress_callback(epoch + 1, epochs, avg_loss);
        }
    }
    
    fn compute_loss(&self, output: &[f64], target: usize) -> f64 {
        if target >= output.len() {
            return 1.0;
        }
        // Cross-entropy loss
        -output[target].ln()
    }
    
    fn update_weights(&mut self, context: &[usize], target: usize, output: &[f64]) {
        // Упрощенный градиентный спуск
        // В реальной реализации здесь был бы полный backpropagation
        let lr = self.learning_rate;
        
        if target >= output.len() || self.layers.is_empty() {
            return;
        }
        
        // Обновление весов выходного слоя
        let output_layer_idx = self.layers.len() - 1;
        if output_layer_idx < self.layers.len() {
            let error = output[target] - 1.0; // gradient
            
            // Простое обновление bias
            if target < self.layers[output_layer_idx].biases.len() {
                self.layers[output_layer_idx].biases[target] -= lr * error;
            }
        }
    }
    
    fn sample_token(&self, probs: &[f64]) -> usize {
        let mut rng = rand::thread_rng();
        let random_val: f64 = rng.gen();
        let mut cumsum = 0.0;
        
        for (idx, &prob) in probs.iter().enumerate() {
            cumsum += prob;
            if random_val < cumsum {
                return idx;
            }
        }
        
        probs.len().saturating_sub(1)
    }
    
    /// Токенизация текста
    pub fn tokenize(&self, text: &str) -> Vec<usize> {
        text.split_whitespace()
            .map(|word| {
                let word_lower = word.to_lowercase();
                *self.vocab.get(&word_lower).unwrap_or(&self.get_unk_token())
            })
            .collect()
    }
    
    fn get_unk_token(&self) -> usize {
        *self.vocab.get("<UNK>").unwrap_or(&0)
    }
    
    /// Декодирование токенов в текст
    pub fn decode(&self, tokens: &[usize]) -> String {
        tokens
            .iter()
            .filter_map(|&token| self.reverse_vocab.get(&token))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Добавление нового слова в словарь
    pub fn add_to_vocab(&mut self, word: String) {
        if !self.vocab.contains_key(&word) {
            let idx = self.vocab.len();
            self.vocab.insert(word.clone(), idx);
            self.reverse_vocab.insert(idx, word);
            
            // Расширяем embedding layer
            let mut rng = rand::thread_rng();
            if !self.layers.is_empty() {
                let new_embedding: Vec<f64> = (0..self.embedding_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect();
                self.layers[0].weights.push(new_embedding);
            }
        }
    }
    
    /// Сохранение модели
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }
    
    /// Загрузка модели
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let model = serde_json::from_str(&data)?;
        Ok(model)
    }
    
    /// Получение информации о модели
    pub fn info(&self) -> String {
        format!(
            "Модель AI (fp64)\n\
             Словарь: {} слов\n\
             Embedding dimension: {}\n\
             Hidden dimension: {}\n\
             Context length: {}\n\
             Слои: {}\n\
             Learning rate: {}",
            self.vocab.len(),
            self.embedding_dim,
            self.hidden_dim,
            self.context_length,
            self.layers.len(),
            self.learning_rate
        )
    }
}

impl Default for AIModel {
    fn default() -> Self {
        Self::new(128, 256, 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_creation() {
        let model = AIModel::new(64, 128, 4);
        assert_eq!(model.embedding_dim, 64);
        assert_eq!(model.hidden_dim, 128);
        assert_eq!(model.context_length, 4);
    }
    
    #[test]
    fn test_tokenization() {
        let model = AIModel::default();
        let tokens = model.tokenize("привет как дела");
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_generation() {
        let model = AIModel::default();
        let response = model.generate("привет", 5);
        assert!(!response.is_empty());
    }
}
