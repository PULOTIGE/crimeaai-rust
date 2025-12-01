//! # CrimeaAI Ecosystem
//!
//! 🧠 AI-экосистема на Rust с терминальным UI

use crimeaai::{Ecosystem, KaifState, voxel::EmotionType};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, Paragraph, Sparkline,
    },
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

fn main() -> Result<(), io::Error> {
    // Баннер
    println!(r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║   ██████╗██████╗ ██╗███╗   ███╗███████╗ █████╗            ║
    ║  ██╔════╝██╔══██╗██║████╗ ████║██╔════╝██╔══██╗           ║
    ║  ██║     ██████╔╝██║██╔████╔██║█████╗  ███████║           ║
    ║  ██║     ██╔══██╗██║██║╚██╔╝██║██╔══╝  ██╔══██║           ║
    ║  ╚██████╗██║  ██║██║██║ ╚═╝ ██║███████╗██║  ██║           ║
    ║   ╚═════╝╚═╝  ╚═╝╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝           ║
    ║                                                           ║
    ║          🧠 AI ECOSYSTEM v1.0 🦀 RUST EDITION             ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
    "#);
    
    println!("Инициализация...");
    
    // Создаём экосистему
    let mut ecosystem = Ecosystem::new(100_000, 1000, 1000);
    
    // Настройка терминала
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Запуск UI
    let res = run_app(&mut terminal, &mut ecosystem);
    
    // Восстановление терминала
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("Error: {:?}", err);
    }
    
    println!("👋 До свидания!");
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ecosystem: &mut Ecosystem,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16); // ~60 FPS
    
    let mut kaif_history: Vec<u64> = vec![0; 100];
    let mut health_history: Vec<u64> = vec![50; 100];
    let mut energy_history: Vec<u64> = vec![50; 100];
    
    loop {
        // Рендер
        terminal.draw(|f| {
            ui(f, ecosystem, &kaif_history, &health_history, &energy_history);
        })?;
        
        // Обработка событий
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char(' ') => ecosystem.toggle_pause(),
                        KeyCode::Char('r') => {
                            *ecosystem = Ecosystem::new(100_000, 1000, 1000);
                            kaif_history = vec![0; 100];
                        }
                        KeyCode::Char('s') => ecosystem.search_concepts(),
                        _ => {}
                    }
                }
            }
        }
        
        // Обновление
        if last_tick.elapsed() >= tick_rate {
            let dt = last_tick.elapsed().as_secs_f32();
            last_tick = Instant::now();
            
            ecosystem.update(dt.min(0.1));
            
            // Обновляем историю
            let stats = ecosystem.get_stats();
            kaif_history.push((stats.kaif * 100.0).min(100.0) as u64);
            health_history.push((stats.avg_health * 100.0) as u64);
            energy_history.push((stats.avg_energy * 100.0) as u64);
            
            if kaif_history.len() > 100 {
                kaif_history.remove(0);
                health_history.remove(0);
                energy_history.remove(0);
            }
        }
    }
}

fn ui(
    f: &mut Frame,
    ecosystem: &Ecosystem,
    kaif_history: &[u64],
    health_history: &[u64],
    energy_history: &[u64],
) {
    let stats = ecosystem.get_stats();
    
    // Основной layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),      // Main
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());
    
    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("🧠 CrimeaAI Ecosystem ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("| "),
            Span::styled(format!("FPS: {:.0}", stats.fps), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Tick: {}", stats.ticks), Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled(
                format!("KAIF: {:.3} ({})", stats.kaif, stats.kaif_state),
                Style::default().fg(kaif_color(&stats.kaif_state))
            ),
            Span::raw(" | "),
            Span::styled(
                if ecosystem.paused { "⏸ PAUSED" } else { "▶ RUNNING" },
                Style::default().fg(if ecosystem.paused { Color::Yellow } else { Color::Green })
            ),
        ])
    ])
    .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(header, chunks[0]);
    
    // Main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Left panel
            Constraint::Percentage(50),  // Center
            Constraint::Percentage(25),  // Right panel
        ])
        .split(chunks[1]);
    
    // Left panel - Stats
    render_stats_panel(f, ecosystem, main_chunks[0]);
    
    // Center - Graphs
    render_graphs(f, kaif_history, health_history, energy_history, main_chunks[1]);
    
    // Right panel - Emotions & Concepts
    render_info_panel(f, ecosystem, main_chunks[2]);
    
    // Footer
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(" [Q] Quit  [Space] Pause  [R] Reset  [S] Search concepts "),
        ])
    ])
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title(" Controls "));
    f.render_widget(footer, chunks[2]);
}

fn render_stats_panel(f: &mut Frame, ecosystem: &Ecosystem, area: Rect) {
    let stats = ecosystem.get_stats();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Stats
            Constraint::Length(5),  // Health gauge
            Constraint::Length(5),  // Energy gauge
            Constraint::Min(0),     // Rest
        ])
        .split(area);
    
    // Stats list
    let items: Vec<ListItem> = vec![
        ListItem::new(format!("🧬 Nucleotides: {}", stats.nucleotide_count)),
        ListItem::new(format!("🌍 Voxels: {}", stats.voxel_count)),
        ListItem::new(format!("📚 Concepts: {}", stats.concept_count)),
        ListItem::new(format!("⏱️ Uptime: {:.1}s", stats.uptime_secs)),
        ListItem::new(format!("💡 Patterns: {}", ecosystem.patterns.count())),
    ];
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" System "))
        .style(Style::default().fg(Color::White));
    f.render_widget(list, chunks[0]);
    
    // Health gauge
    let health_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Health "))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((stats.avg_health * 100.0) as u16)
        .label(format!("{:.0}%", stats.avg_health * 100.0));
    f.render_widget(health_gauge, chunks[1]);
    
    // Energy gauge
    let energy_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Energy "))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent((stats.avg_energy * 100.0) as u16)
        .label(format!("{:.0}%", stats.avg_energy * 100.0));
    f.render_widget(energy_gauge, chunks[2]);
}

fn render_graphs(
    f: &mut Frame,
    kaif_history: &[u64],
    health_history: &[u64],
    energy_history: &[u64],
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Kaif
            Constraint::Percentage(30), // Health
            Constraint::Percentage(30), // Energy
        ])
        .split(area);
    
    // Kaif sparkline
    let kaif_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" ⚡ Kaif "))
        .data(kaif_history)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(kaif_sparkline, chunks[0]);
    
    // Health sparkline
    let health_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" 💚 Health "))
        .data(health_history)
        .style(Style::default().fg(Color::Green));
    f.render_widget(health_sparkline, chunks[1]);
    
    // Energy sparkline
    let energy_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" ⚡ Energy "))
        .data(energy_history)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(energy_sparkline, chunks[2]);
}

fn render_info_panel(f: &mut Frame, ecosystem: &Ecosystem, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Emotions
            Constraint::Percentage(50), // Concepts
        ])
        .split(area);
    
    // Emotions
    let emotions = ecosystem.voxels.get_emotion_distribution();
    let emotion_names = ["Joy", "Sad", "Anger", "Fear", "Surprise", "Disgust", "Curious", "Peace"];
    let emotion_colors = [
        Color::Yellow, Color::Blue, Color::Red, Color::Magenta,
        Color::Rgb(255, 150, 0), Color::Green, Color::Cyan, Color::Gray
    ];
    
    let emotion_items: Vec<ListItem> = emotion_names.iter()
        .enumerate()
        .map(|(i, name)| {
            let value = emotions[i];
            let bar_len = (value * 20.0) as usize;
            let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            ListItem::new(format!("{:8} {} {:.0}%", name, bar, value * 100.0))
                .style(Style::default().fg(emotion_colors[i]))
        })
        .collect();
    
    let emotions_list = List::new(emotion_items)
        .block(Block::default().borders(Borders::ALL).title(" 🎭 Emotions "));
    f.render_widget(emotions_list, chunks[0]);
    
    // Concepts
    let top_concepts = ecosystem.concepts.top_concepts(8);
    let concept_items: Vec<ListItem> = top_concepts.iter()
        .map(|c| ListItem::new(format!("• {} ({:.2})", c.term, c.importance)))
        .collect();
    
    let concepts_list = List::new(concept_items)
        .block(Block::default().borders(Borders::ALL).title(" 📚 Concepts "))
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(concepts_list, chunks[1]);
}

fn kaif_color(state: &str) -> Color {
    match state {
        "dormant" => Color::DarkGray,
        "calm" => Color::Blue,
        "active" => Color::Green,
        "excited" => Color::Yellow,
        "ecstatic" => Color::Magenta,
        _ => Color::White,
    }
}
