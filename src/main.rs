mod column;
mod config;
mod input;
mod renderer;

use std::io::{Stdout, stdout};
use std::thread;
use std::time::Instant;

use column::{Column, ColumnSettings};
use config::{AppConfig, ConfigError};
use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use input::{InputAction, poll_input};
use rand::thread_rng;
use renderer::Renderer;

struct TerminalGuard {
    stdout: Stdout,
}

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;
        stdout.execute(Clear(ClearType::All))?;
        Ok(Self { stdout })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.stdout.execute(Show);
        let _ = self.stdout.execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn build_columns(width: u16, density: f32, settings: ColumnSettings) -> Vec<Column> {
    let mut rng = thread_rng();
    let mut columns = Vec::with_capacity(width as usize);
    for x in 0..width {
        if rand::Rng::gen_bool(&mut rng, density as f64) {
            columns.push(Column::new(x, settings, &mut rng));
        }
    }
    if columns.is_empty() && width > 0 {
        columns.push(Column::new(0, settings, &mut rng));
    }
    columns
}

fn run(guard: &mut TerminalGuard, config: AppConfig) -> std::io::Result<()> {
    let (mut width, mut height) = terminal::size()?;
    if width == 0 || height == 0 {
        return Ok(());
    }

    let mut columns = build_columns(width, config.density, config.column_settings());
    let mut renderer = Renderer::new(width, height);
    let mut rng = thread_rng();
    let frame_time = config.frame_time();
    let mut paused = false;

    loop {
        let frame_start = Instant::now();

        loop {
            match poll_input()? {
                InputAction::None => break,
                InputAction::Quit => return Ok(()),
                InputAction::TogglePause => paused = !paused,
                InputAction::Resized(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    columns = build_columns(width, config.density, config.column_settings());
                    renderer.resize(width, height);
                }
            }
        }

        if !paused {
            for column in &mut columns {
                column.tick(height, &mut rng);
            }
        }
        renderer.render(&mut guard.stdout, &columns)?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
    }
}

fn main() -> std::io::Result<()> {
    let config = match AppConfig::from_args() {
        Ok(cfg) => cfg,
        Err(ConfigError::Help) => {
            println!("{}", config::usage_text());
            return Ok(());
        }
        Err(err) => {
            eprintln!("{err}");
            return Ok(());
        }
    };
    let mut guard = TerminalGuard::new()?;
    run(&mut guard, config)
}
