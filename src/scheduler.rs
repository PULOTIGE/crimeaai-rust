//! # Scheduler - Планировщик задач

use std::time::{Duration, Instant};

/// Приоритет задачи
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

/// Запланированная задача
pub struct ScheduledTask {
    pub name: String,
    pub interval: Duration,
    pub priority: TaskPriority,
    pub enabled: bool,
    pub last_run: Instant,
    pub run_count: u64,
    pub total_time: Duration,
}

impl ScheduledTask {
    pub fn new(name: &str, interval_secs: f32, priority: TaskPriority) -> Self {
        Self {
            name: name.to_string(),
            interval: Duration::from_secs_f32(interval_secs),
            priority,
            enabled: true,
            last_run: Instant::now(),
            run_count: 0,
            total_time: Duration::ZERO,
        }
    }
    
    pub fn should_run(&self) -> bool {
        self.enabled && self.last_run.elapsed() >= self.interval
    }
    
    pub fn record_run(&mut self, duration: Duration) {
        self.last_run = Instant::now();
        self.run_count += 1;
        self.total_time += duration;
    }
    
    pub fn avg_time_ms(&self) -> f32 {
        if self.run_count == 0 {
            0.0
        } else {
            self.total_time.as_secs_f32() * 1000.0 / self.run_count as f32
        }
    }
}

/// Планировщик
pub struct Scheduler {
    pub tasks: Vec<ScheduledTask>,
    pub running: bool,
    pub paused: bool,
    pub start_time: Instant,
    pub total_ticks: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            running: false,
            paused: false,
            start_time: Instant::now(),
            total_ticks: 0,
        }
    }
    
    pub fn add_task(&mut self, name: &str, interval_secs: f32, priority: TaskPriority) {
        self.tasks.push(ScheduledTask::new(name, interval_secs, priority));
    }
    
    pub fn enable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = true;
        }
    }
    
    pub fn disable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = false;
        }
    }
    
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
