//! # Concept Search - Поиск концептов
//!
//! Поиск и извлечение концептов из интернета.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Концепт - единица знания
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub term: String,
    pub definition: String,
    pub source_url: String,
    pub importance: f32,
    pub discovery_time: f64,
    pub access_count: u32,
}

impl Concept {
    pub fn new(term: String) -> Self {
        Self {
            term,
            definition: String::new(),
            source_url: String::new(),
            importance: 1.0,
            discovery_time: 0.0,
            access_count: 0,
        }
    }
}

/// Извлечение терминов из текста
pub fn extract_terms(text: &str) -> Vec<String> {
    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been",
        "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "must", "shall", "can",
        "this", "that", "these", "those", "it", "its", "with", "for",
        "from", "into", "onto", "about", "above", "below", "between",
    ].into_iter().collect();
    
    let mut terms = Vec::new();
    
    for word in text.split_whitespace() {
        let clean: String = word.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        
        if clean.len() >= 3 && clean.len() <= 30 && !stop_words.contains(clean.as_str()) {
            terms.push(clean);
        }
    }
    
    // Убираем дубликаты
    terms.sort();
    terms.dedup();
    
    terms
}

/// Поисковик концептов
pub struct ConceptSearcher {
    pub concepts: HashMap<String, Concept>,
    pub base_keywords: Vec<String>,
    pub total_searches: u32,
    pub last_search_time: f64,
}

impl ConceptSearcher {
    pub fn new(keywords: Vec<String>) -> Self {
        Self {
            concepts: HashMap::new(),
            base_keywords: keywords,
            total_searches: 0,
            last_search_time: 0.0,
        }
    }
    
    /// Симуляция поиска (без реальных HTTP запросов)
    pub fn search_simulated(&mut self) -> Vec<Concept> {
        self.total_searches += 1;
        
        let simulated_terms = [
            "neural architecture", "deep learning", "gradient descent",
            "backpropagation", "attention mechanism", "transformer model",
            "convolutional network", "recurrent network", "generative model",
            "reinforcement learning", "policy gradient", "value function",
            "embedding space", "latent representation", "feature extraction",
            "batch normalization", "dropout regularization", "weight decay",
        ];
        
        let mut rng = rand::thread_rng();
        use rand::Rng;
        
        let count = rng.gen_range(3..8);
        let mut results = Vec::new();
        
        for _ in 0..count {
            let idx = rng.gen_range(0..simulated_terms.len());
            let term = simulated_terms[idx].to_string();
            
            if !self.concepts.contains_key(&term) {
                let concept = Concept {
                    term: term.clone(),
                    definition: format!("Simulated concept: {}", term),
                    source_url: String::new(),
                    importance: rng.gen_range(0.3..1.0),
                    discovery_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    access_count: 0,
                };
                
                self.concepts.insert(term.clone(), concept.clone());
                results.push(concept);
            }
        }
        
        results
    }
    
    /// Реальный поиск через DuckDuckGo (блокирующий)
    #[cfg(feature = "web_search")]
    pub fn search_real(&mut self, query: &str) -> Vec<Concept> {
        use scraper::{Html, Selector};
        
        self.total_searches += 1;
        let mut results = Vec::new();
        
        let url = format!("https://duckduckgo.com/html/?q={}", query);
        
        match reqwest::blocking::Client::new()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
        {
            Ok(response) => {
                if let Ok(body) = response.text() {
                    let document = Html::parse_document(&body);
                    let selector = Selector::parse("a.result__a").unwrap();
                    
                    for (i, element) in document.select(&selector).enumerate() {
                        if i >= 5 { break; }
                        
                        let title = element.text().collect::<String>();
                        let terms = extract_terms(&title);
                        
                        for term in terms.into_iter().take(3) {
                            if !self.concepts.contains_key(&term) {
                                let concept = Concept {
                                    term: term.clone(),
                                    definition: title.clone(),
                                    source_url: element.value().attr("href").unwrap_or("").to_string(),
                                    importance: 1.0 - (i as f32 * 0.1),
                                    discovery_time: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs_f64(),
                                    access_count: 0,
                                };
                                
                                self.concepts.insert(term, concept.clone());
                                results.push(concept);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Search error: {}", e);
            }
        }
        
        results
    }
    
    pub fn get_concept(&mut self, term: &str) -> Option<&mut Concept> {
        if let Some(c) = self.concepts.get_mut(term) {
            c.access_count += 1;
            Some(c)
        } else {
            None
        }
    }
    
    pub fn top_concepts(&self, n: usize) -> Vec<&Concept> {
        let mut sorted: Vec<&Concept> = self.concepts.values().collect();
        sorted.sort_by(|a, b| {
            let score_a = a.importance * (1.0 + a.access_count as f32 * 0.1);
            let score_b = b.importance * (1.0 + b.access_count as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap()
        });
        sorted.truncate(n);
        sorted
    }
    
    pub fn count(&self) -> usize {
        self.concepts.len()
    }
}

impl Default for ConceptSearcher {
    fn default() -> Self {
        Self::new(vec![
            "AI".to_string(),
            "neural network".to_string(),
            "machine learning".to_string(),
        ])
    }
}
