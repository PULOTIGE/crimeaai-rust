use crate::ai_model::AIModel;
use crate::file_processor::{FileProcessor, FileStats};
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Режим работы приложения
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Chat,
    Training,
}

/// Сообщение в чате
#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub is_user: bool,
    pub timestamp: String,
}

/// Статус обучения
#[derive(Clone)]
pub struct TrainingStatus {
    pub is_training: bool,
    pub current_epoch: usize,
    pub total_epochs: usize,
    pub loss: f64,
    pub progress: f32,
}

/// Основной UI чат-приложения
pub struct ChatUI {
    // Модель AI
    pub model: Arc<Mutex<AIModel>>,
    
    // Обработчик файлов
    pub file_processor: FileProcessor,
    
    // Режим работы
    pub mode: AppMode,
    
    // Чат
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    
    // Обучение
    pub training_status: TrainingStatus,
    pub training_data: Vec<String>,
    pub epochs: usize,
    pub loaded_files: Vec<(PathBuf, String)>,
    pub file_stats: Option<FileStats>,
    
    // UI состояние
    pub show_model_info: bool,
    pub auto_scroll: bool,
    pub file_path_input: String,
}

impl ChatUI {
    pub fn new() -> Self {
        let model = AIModel::default();
        
        // Приветственное сообщение
        let welcome_msg = ChatMessage {
            text: "Привет! Я AI ассистент с возможностью дообучения. Выберите режим работы:\n\
                   • Разговор - для общения\n\
                   • Обучение - для загрузки файлов и дообучения модели".to_string(),
            is_user: false,
            timestamp: Self::get_timestamp(),
        };
        
        Self {
            model: Arc::new(Mutex::new(model)),
            file_processor: FileProcessor::new(),
            mode: AppMode::Chat,
            messages: vec![welcome_msg],
            input_text: String::new(),
            training_status: TrainingStatus {
                is_training: false,
                current_epoch: 0,
                total_epochs: 0,
                loss: 0.0,
                progress: 0.0,
            },
            training_data: Vec::new(),
            epochs: 10,
            loaded_files: Vec::new(),
            file_stats: None,
            show_model_info: false,
            auto_scroll: true,
            file_path_input: String::new(),
        }
    }
    
    fn get_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;
        format!("{:02}:{:02}", hours, minutes)
    }
    
    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        
        // Добавляем сообщение пользователя
        let user_msg = ChatMessage {
            text: self.input_text.clone(),
            is_user: true,
            timestamp: Self::get_timestamp(),
        };
        self.messages.push(user_msg);
        
        // Генерируем ответ
        let input = self.input_text.clone();
        self.input_text.clear();
        
        let model = self.model.clone();
        let response = {
            let model = model.lock().unwrap();
            model.generate(&input, 50)
        };
        
        // Если ответ пустой, даем стандартный ответ
        let response_text = if response.trim().is_empty() {
            "Я пока не знаю, как на это ответить. Попробуйте дообучить меня на ваших данных!".to_string()
        } else {
            response
        };
        
        let ai_msg = ChatMessage {
            text: response_text,
            is_user: false,
            timestamp: Self::get_timestamp(),
        };
        self.messages.push(ai_msg);
    }
    
    fn load_file(&mut self) {
        let path = PathBuf::from(&self.file_path_input);
        
        match self.file_processor.read_file(&path) {
            Ok(content) => {
                self.file_stats = Some(self.file_processor.get_file_stats(&content));
                self.loaded_files.push((path.clone(), content.clone()));
                
                let training_examples = self.file_processor.extract_training_data(&content);
                self.training_data.extend(training_examples);
                
                self.messages.push(ChatMessage {
                    text: format!("✓ Файл загружен: {:?}\n{}", 
                        path.file_name().unwrap_or_default(),
                        self.file_stats.as_ref().unwrap().format()
                    ),
                    is_user: false,
                    timestamp: Self::get_timestamp(),
                });
                
                self.file_path_input.clear();
            }
            Err(e) => {
                self.messages.push(ChatMessage {
                    text: format!("✗ Ошибка загрузки файла: {}", e),
                    is_user: false,
                    timestamp: Self::get_timestamp(),
                });
            }
        }
    }
    
    fn start_training(&mut self) {
        if self.training_data.is_empty() {
            self.messages.push(ChatMessage {
                text: "✗ Нет данных для обучения. Загрузите файлы!".to_string(),
                is_user: false,
                timestamp: Self::get_timestamp(),
            });
            return;
        }
        
        if let Err(e) = self.file_processor.validate_training_data(&self.training_data) {
            self.messages.push(ChatMessage {
                text: format!("✗ Ошибка валидации: {}", e),
                is_user: false,
                timestamp: Self::get_timestamp(),
            });
            return;
        }
        
        self.training_status.is_training = true;
        self.training_status.total_epochs = self.epochs;
        self.training_status.current_epoch = 0;
        
        self.messages.push(ChatMessage {
            text: format!("🚀 Начинаю обучение на {} примерах, {} эпох...", 
                self.training_data.len(), self.epochs),
            is_user: false,
            timestamp: Self::get_timestamp(),
        });
        
        // Запускаем обучение в отдельном потоке
        let model = self.model.clone();
        let data = self.training_data.clone();
        let epochs = self.epochs;
        
        thread::spawn(move || {
            let mut model = model.lock().unwrap();
            
            model.train(&data, epochs, |epoch, total, loss| {
                // Прогресс обучения
                println!("Эпоха {}/{}, Loss: {:.4}", epoch, total, loss);
            });
        });
    }
}

impl eframe::App for ChatUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Устанавливаем белую тему
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
        style.visuals.panel_fill = egui::Color32::from_rgb(250, 250, 250);
        ctx.set_style(style);
        
        // Верхняя панель с режимами
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                
                ui.heading("🤖 AI Ассистент");
                
                ui.add_space(40.0);
                
                ui.selectable_value(&mut self.mode, AppMode::Chat, "💬 Разговор");
                ui.selectable_value(&mut self.mode, AppMode::Training, "📚 Обучение");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    if ui.button("ℹ️ Инфо").clicked() {
                        self.show_model_info = !self.show_model_info;
                    }
                });
            });
            ui.add_space(10.0);
            ui.separator();
        });
        
        // Основная панель
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Chat => self.render_chat_mode(ui),
                AppMode::Training => self.render_training_mode(ui),
            }
        });
        
        // Информация о модели
        if self.show_model_info {
            egui::Window::new("Информация о модели")
                .open(&mut self.show_model_info)
                .show(ctx, |ui| {
                    let model = self.model.lock().unwrap();
                    ui.label(model.info());
                    
                    ui.separator();
                    ui.label(format!("Загружено файлов: {}", self.loaded_files.len()));
                    ui.label(format!("Примеров для обучения: {}", self.training_data.len()));
                });
        }
        
        ctx.request_repaint();
    }
}

impl ChatUI {
    fn render_chat_mode(&mut self, ui: &mut egui::Ui) {
        // Область сообщений
        egui::ScrollArea::vertical()
            .id_source("chat_scroll")
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                ui.add_space(10.0);
                
                for msg in &self.messages {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        
                        let frame_color = if msg.is_user {
                            egui::Color32::from_rgb(230, 240, 255)
                        } else {
                            egui::Color32::from_rgb(245, 245, 245)
                        };
                        
                        let max_width = ui.available_width() - 100.0;
                        
                        egui::Frame::none()
                            .fill(frame_color)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_max_width(max_width);
                                
                                ui.horizontal(|ui| {
                                    let icon = if msg.is_user { "👤" } else { "🤖" };
                                    ui.label(egui::RichText::new(icon).size(16.0));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(&msg.timestamp)
                                                .size(10.0)
                                                .color(egui::Color32::GRAY)
                                        );
                                    });
                                });
                                
                                ui.add_space(4.0);
                                ui.label(&msg.text);
                            });
                    });
                    
                    ui.add_space(10.0);
                }
            });
        
        // Нижняя панель ввода
        egui::TopBottomPanel::bottom("input_panel").show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                
                let text_edit = egui::TextEdit::singleline(&mut self.input_text)
                    .hint_text("Введите сообщение...")
                    .desired_width(ui.available_width() - 100.0);
                
                let response = ui.add(text_edit);
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.send_message();
                    response.request_focus();
                }
                
                if ui.button("📤 Отправить").clicked() {
                    self.send_message();
                }
                
                ui.add_space(20.0);
            });
            ui.add_space(10.0);
        });
    }
    
    fn render_training_mode(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.heading("Дообучение модели");
        });
        
        ui.add_space(20.0);
        
        // Секция загрузки файлов
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .rounding(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - 40.0);
                    
                    ui.label(egui::RichText::new("📁 Загрузка файлов").strong());
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Путь к файлу:");
                        ui.text_edit_singleline(&mut self.file_path_input);
                        
                        if ui.button("📂 Загрузить").clicked() {
                            self.load_file();
                        }
                    });
                    
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(format!("Поддерживаемые форматы: {}", 
                            self.file_processor.supported_extensions.join(", ")))
                            .size(11.0)
                            .color(egui::Color32::GRAY)
                    );
                    
                    if !self.loaded_files.is_empty() {
                        ui.add_space(10.0);
                        ui.label(format!("Загружено файлов: {}", self.loaded_files.len()));
                        
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for (path, _) in &self.loaded_files {
                                    ui.label(format!("  • {:?}", path.file_name().unwrap_or_default()));
                                }
                            });
                    }
                });
        });
        
        ui.add_space(20.0);
        
        // Секция параметров обучения
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .rounding(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - 40.0);
                    
                    ui.label(egui::RichText::new("⚙️ Параметры обучения").strong());
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Количество эпох:");
                        ui.add(egui::Slider::new(&mut self.epochs, 1..=100));
                    });
                    
                    ui.add_space(5.0);
                    ui.label(format!("Примеров для обучения: {}", self.training_data.len()));
                    
                    ui.add_space(10.0);
                    
                    if self.training_status.is_training {
                        ui.label("🔄 Обучение в процессе...");
                        ui.add(egui::ProgressBar::new(self.training_status.progress)
                            .text(format!("Эпоха {}/{}", 
                                self.training_status.current_epoch,
                                self.training_status.total_epochs)));
                    } else {
                        if ui.button("🚀 Начать обучение").clicked() {
                            self.start_training();
                        }
                    }
                });
        });
        
        ui.add_space(20.0);
        
        // Лог обучения
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .rounding(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - 40.0);
                    
                    ui.label(egui::RichText::new("📋 Журнал").strong());
                    ui.add_space(10.0);
                    
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for msg in self.messages.iter().rev().take(10).rev() {
                                if !msg.is_user {
                                    ui.label(format!("[{}] {}", msg.timestamp, msg.text));
                                    ui.add_space(5.0);
                                }
                            }
                        });
                });
        });
    }
}

impl Default for ChatUI {
    fn default() -> Self {
        Self::new()
    }
}
