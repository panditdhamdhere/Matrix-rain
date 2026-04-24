mod column;
mod input;
mod renderer;

use std::io::{Stdout, stdout};
use std::thread;
use std::time::{Duration, Instant};

use column::Column;
use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use input::{InputAction, poll_input};
use rand::thread_rng;
use renderer::Renderer;

const FRAME_TIME: Duration = Duration::from_millis(33);

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

fn build_columns(width: u16) -> Vec<Column> {
    let mut rng = thread_rng();
    (0..width).map(|x| Column::new(x, &mut rng)).collect()
}

fn run(guard: &mut TerminalGuard) -> std::io::Result<()> {
    let (mut width, mut height) = terminal::size()?;
    if width == 0 || height == 0 {
        return Ok(());
    }

    let mut columns = build_columns(width);
    let mut renderer = Renderer::new(width, height);
    let mut rng = thread_rng();

    loop {
        let frame_start = Instant::now();

        loop {
            match poll_input()? {
                InputAction::None => break,
                InputAction::Quit => return Ok(()),
                InputAction::Resized(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    columns = build_columns(width);
                    renderer.resize(width, height);
                }
            }
        }

        for column in &mut columns {
            column.tick(height, &mut rng);
        }
        renderer.render(&mut guard.stdout, &columns)?;

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut guard = TerminalGuard::new()?;
    run(&mut guard)
}
