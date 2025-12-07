use crate::ai_model::AIModel;
use crate::file_processor::{FileProcessor, FileStats};
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

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

/// Основной UI чат-приложения (стиль DeepSeek)
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
            text: "Привет! Я AI ассистент с возможностью дообучения 🤖\n\nВыберите режим:\n• 💬 Разговор - общение со мной\n• 📚 Обучение - загрузка файлов и дообучение\n\nЯ здесь, чтобы помочь!".to_string(),
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
            "Я пока не знаю, как на это ответить. Попробуйте дообучить меня на ваших данных! 📚".to_string()
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
        if self.file_path_input.trim().is_empty() {
            self.messages.push(ChatMessage {
                text: "✗ Ошибка: введите путь к файлу".to_string(),
                is_user: false,
                timestamp: Self::get_timestamp(),
            });
            return;
        }
        
        let path = PathBuf::from(self.file_path_input.trim());
        
        // Проверяем существование файла
        if !path.exists() {
            self.messages.push(ChatMessage {
                text: format!("✗ Файл не найден: {:?}\n\n💡 Попробуйте:\n• examples/training_data_ru.txt\n• examples\\training_data_ru.txt\n• Полный путь к файлу", path),
                is_user: false,
                timestamp: Self::get_timestamp(),
            });
            return;
        }
        
        match self.file_processor.read_file(&path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    self.messages.push(ChatMessage {
                        text: format!("⚠️ Файл пустой!\n\n📁 Файл: {:?}\n\n💡 Убедитесь, что файл содержит текст.", 
                            path.file_name().unwrap_or_default()
                        ),
                        is_user: false,
                        timestamp: Self::get_timestamp(),
                    });
                    return;
                }
                
                self.file_stats = Some(self.file_processor.get_file_stats(&content));
                self.loaded_files.push((path.clone(), content.clone()));
                
                let training_examples = self.file_processor.extract_training_data(&content);
                let examples_count = training_examples.len();
                
                if training_examples.is_empty() {
                    self.messages.push(ChatMessage {
                        text: format!("⚠️ Не удалось извлечь данные для обучения!\n\n📁 Файл: {:?}\n{}\n\n💡 Файл загружен, но текст слишком короткий.\nДобавьте больше содержимого (минимум 5 символов).", 
                            path.file_name().unwrap_or_default(),
                            self.file_stats.as_ref().unwrap().format()
                        ),
                        is_user: false,
                        timestamp: Self::get_timestamp(),
                    });
                    return;
                }
                
                self.training_data.extend(training_examples);
                
                self.messages.push(ChatMessage {
                    text: format!("✅ Файл успешно загружен!\n\n📁 Файл: {:?}\n{}\n📊 Извлечено примеров: {}\n\n💡 Теперь нажмите \"Начать обучение\"!", 
                        path.file_name().unwrap_or_default(),
                        self.file_stats.as_ref().unwrap().format(),
                        examples_count
                    ),
                    is_user: false,
                    timestamp: Self::get_timestamp(),
                });
                
                self.file_path_input.clear();
            }
            Err(e) => {
                self.messages.push(ChatMessage {
                    text: format!("❌ Ошибка загрузки файла!\n\n{}\n\n💡 Проверьте:\n• Путь к файлу правильный?\n• Файл существует?\n• Формат поддерживается?", e),
                    is_user: false,
                    timestamp: Self::get_timestamp(),
                });
            }
        }
    }
    
    fn start_training(&mut self) {
        if self.training_data.is_empty() {
            self.messages.push(ChatMessage {
                text: "✗ Нет данных для обучения. Загрузите файлы! 📁".to_string(),
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
            text: format!("🚀 Начинаю обучение!\n\n📊 Примеров: {}\n🔄 Эпох: {}\n\nПодождите...", 
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
                println!("Эпоха {}/{}, Loss: {:.4}", epoch, total, loss);
            });
        });
    }
}

impl eframe::App for ChatUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Устанавливаем стиль DeepSeek - голубые оттенки
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        
        // Голубые оттенки
        style.visuals.window_fill = egui::Color32::from_rgb(250, 252, 255);  // Очень светло-голубой фон
        style.visuals.panel_fill = egui::Color32::from_rgb(245, 250, 255);   // Светло-голубая панель
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(230, 242, 255); // Голубой акцент
        
        // Закругленные углы
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.menu_rounding = egui::Rounding::same(6.0);
        
        ctx.set_style(style);
        
        // Верхняя панель с режимами (компактная)
        egui::TopBottomPanel::top("top_panel")
            .min_height(50.0)
            .show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                
                // Заголовок
                ui.label(egui::RichText::new("🤖 AI Ассистент").size(18.0).strong());
                
                ui.add_space(20.0);
                
                // Режимы
                let chat_selected = self.mode == AppMode::Chat;
                let train_selected = self.mode == AppMode::Training;
                
                if ui.selectable_label(chat_selected, 
                    egui::RichText::new("💬 Разговор").size(14.0))
                    .clicked() {
                    self.mode = AppMode::Chat;
                }
                
                if ui.selectable_label(train_selected, 
                    egui::RichText::new("📚 Обучение").size(14.0))
                    .clicked() {
                    self.mode = AppMode::Training;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.button(egui::RichText::new("ℹ️").size(16.0)).clicked() {
                        self.show_model_info = !self.show_model_info;
                    }
                });
            });
            ui.add_space(5.0);
        });
        
        // Нижняя панель ввода (фиксированная, как у DeepSeek)
        egui::TopBottomPanel::bottom("input_panel")
            .min_height(70.0)
            .show(ctx, |ui| {
            ui.add_space(10.0);
            
            // Панель ввода с голубой рамкой
            egui::Frame::none()
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width() - 20.0);
                    
                    ui.horizontal(|ui| {
                        // Поле ввода
                        let text_edit = egui::TextEdit::multiline(&mut self.input_text)
                            .hint_text("Напишите сообщение...")
                            .desired_width(ui.available_width() - 60.0)
                            .desired_rows(1)
                            .frame(false);
                        
                        let response = ui.add(text_edit);
                        
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !ui.input(|i| i.modifiers.shift) {
                                self.send_message();
                                response.request_focus();
                            }
                        }
                        
                        ui.add_space(5.0);
                        
                        // Кнопка отправки (голубая)
                        let send_button = egui::Button::new(egui::RichText::new("📤").size(20.0))
                            .fill(egui::Color32::from_rgb(100, 150, 255));
                        
                        if ui.add(send_button).clicked() {
                            self.send_message();
                        }
                    });
                });
            
            ui.add_space(10.0);
        });
        
        // Центральная панель с контентом
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Chat => self.render_chat_mode(ui),
                AppMode::Training => self.render_training_mode(ui),
            }
        });
        
        // Окно информации о модели
        if self.show_model_info {
            egui::Window::new("ℹ️ Информация о модели")
                .open(&mut self.show_model_info)
                .resizable(false)
                .show(ctx, |ui| {
                    let model = self.model.lock().unwrap();
                    ui.label(model.info());
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    ui.label(format!("📁 Загружено файлов: {}", self.loaded_files.len()));
                    ui.label(format!("📊 Примеров для обучения: {}", self.training_data.len()));
                });
        }
        
        ctx.request_repaint();
    }
}

impl ChatUI {
    fn render_chat_mode(&mut self, ui: &mut egui::Ui) {
        // Область сообщений с auto-scroll
        egui::ScrollArea::vertical()
            .id_source("chat_scroll")
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(10.0);
                
                for msg in &self.messages {
                    let available_width = ui.available_width();
                    let max_width = available_width * 0.75;  // 75% ширины экрана
                    
                    if msg.is_user {
                        // Сообщение пользователя справа с голубым фоном
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.add_space(10.0);
                            
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(220, 235, 255))  // Голубой фон
                                .rounding(egui::Rounding::same(12.0))
                                .inner_margin(egui::Margin::same(12.0))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 210, 255)))
                                .show(ui, |ui| {
                                    ui.set_max_width(max_width);
                                    
                                    ui.label(
                                        egui::RichText::new(&msg.timestamp)
                                            .size(10.0)
                                            .color(egui::Color32::DARK_GRAY)
                                    );
                                    
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(&msg.text).size(14.0));
                                });
                        });
                    } else {
                        // Сообщение AI слева с белым фоном
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.add_space(10.0);
                            
                            egui::Frame::none()
                                .fill(egui::Color32::WHITE)
                                .rounding(egui::Rounding::same(12.0))
                                .inner_margin(egui::Margin::same(12.0))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
                                .show(ui, |ui| {
                                    ui.set_max_width(max_width);
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🤖").size(16.0));
                                        ui.label(
                                            egui::RichText::new(&msg.timestamp)
                                                .size(10.0)
                                                .color(egui::Color32::DARK_GRAY)
                                        );
                                    });
                                    
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(&msg.text).size(14.0));
                                });
                        });
                    }
                    
                    ui.add_space(12.0);
                }
                
                ui.add_space(20.0);  // Отступ снизу
            });
    }
    
    fn render_training_mode(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(15.0);
                
                // Секция загрузки файлов
                egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 220, 240)))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width() - 30.0);
                        
                        ui.label(egui::RichText::new("📁 Загрузка файлов").size(16.0).strong());
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Путь к файлу:");
                            
                            let text_edit = egui::TextEdit::singleline(&mut self.file_path_input)
                                .hint_text("examples/training_data_ru.txt")
                                .desired_width(ui.available_width() - 120.0);
                            ui.add(text_edit);
                            
                            let load_button = egui::Button::new("📂 Загрузить")
                                .fill(egui::Color32::from_rgb(100, 150, 255));
                            
                            if ui.add(load_button).clicked() {
                                self.load_file();
                            }
                        });
                        
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("Форматы: {}", 
                                self.file_processor.supported_extensions.join(", ")))
                                .size(11.0)
                                .color(egui::Color32::GRAY)
                        );
                        
                        if !self.loaded_files.is_empty() {
                            ui.add_space(10.0);
                            ui.label(format!("✓ Загружено: {} файлов", self.loaded_files.len()));
                        }
                    });
                
                ui.add_space(15.0);
                
                // Секция параметров обучения
                egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 220, 240)))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width() - 30.0);
                        
                        ui.label(egui::RichText::new("⚙️ Параметры обучения").size(16.0).strong());
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Количество эпох:");
                            ui.add(egui::Slider::new(&mut self.epochs, 1..=100).text("эпох"));
                        });
                        
                        ui.add_space(5.0);
                        ui.label(format!("📊 Примеров: {}", self.training_data.len()));
                        
                        ui.add_space(10.0);
                        
                        if self.training_status.is_training {
                            ui.label("🔄 Обучение в процессе...");
                            ui.add(egui::ProgressBar::new(self.training_status.progress)
                                .text(format!("Эпоха {}/{}", 
                                    self.training_status.current_epoch,
                                    self.training_status.total_epochs)));
                        } else {
                            let train_button = egui::Button::new(
                                egui::RichText::new("🚀 Начать обучение").size(14.0))
                                .fill(egui::Color32::from_rgb(100, 180, 100));
                            
                            if ui.add(train_button).clicked() {
                                self.start_training();
                            }
                        }
                    });
                
                ui.add_space(15.0);
                
                // Журнал
                egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 220, 240)))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width() - 30.0);
                        
                        ui.label(egui::RichText::new("📋 Журнал").size(16.0).strong());
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
                
                ui.add_space(20.0);
            });
    }
}

impl Default for ChatUI {
    fn default() -> Self {
        Self::new()
    }
}
