mod ai_model;
mod file_processor;
mod chat_ui;

fn main() -> Result<(), eframe::Error> {
    use chat_ui::ChatUI;

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::epaint::Vec2::new(1200.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "AI Ассистент",
        options,
        Box::new(|_cc| Box::new(ChatUI::new())),
    )
}
