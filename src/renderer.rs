use std::collections::{HashMap, HashSet};
use std::io::{Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};

use crate::column::Column;

#[derive(Clone, Copy, PartialEq, Eq)]
struct CellStyle {
    glyph: char,
    color: Color,
    bold: bool,
}

pub struct Renderer {
    width: u16,
    height: u16,
    previous: HashMap<(u16, u16), CellStyle>,
    frame: u64,
}

impl Renderer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            previous: HashMap::new(),
            frame: 0,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.previous.clear();
    }

    pub fn render(&mut self, stdout: &mut Stdout, columns: &[Column]) -> std::io::Result<()> {
        let mut current = HashMap::new();
        self.frame = self.frame.wrapping_add(1);

        for column in columns {
            if !column.active {
                continue;
            }
            for i in 0..column.trail_length {
                let y = column.y.floor() as i32 - i as i32;
                if y < 0 || y >= self.height as i32 || column.x >= self.width {
                    continue;
                }
                let Some(&glyph) = column.trail.get(i) else {
                    continue;
                };
                let (base_color, bold) = color_for_index(i, column.trail_length);
                let color = apply_pulse(base_color, column.x, y as u16, self.frame, i);
                current.insert((column.x, y as u16), CellStyle { glyph, color, bold });
            }
        }

        let mut dirty = HashSet::new();
        for key in self.previous.keys() {
            dirty.insert(*key);
        }
        for key in current.keys() {
            dirty.insert(*key);
        }

        for (x, y) in dirty {
            let prev = self.previous.get(&(x, y));
            let next = current.get(&(x, y));
            if prev == next {
                continue;
            }

            stdout.queue(MoveTo(x, y))?;
            match next {
                Some(style) => {
                    stdout
                        .queue(SetForegroundColor(style.color))?
                        .queue(SetAttribute(if style.bold {
                            Attribute::Bold
                        } else {
                            Attribute::NormalIntensity
                        }))?
                        .queue(Print(style.glyph))?;
                }
                None => {
                    stdout
                        .queue(SetForegroundColor(Color::Rgb { r: 0, g: 0, b: 0 }))?
                        .queue(SetAttribute(Attribute::NormalIntensity))?
                        .queue(Print(' '))?;
                }
            }
        }

        stdout.flush()?;
        self.previous = current;
        Ok(())
    }
}

fn color_for_index(index: usize, trail_length: usize) -> (Color, bool) {
    if index == 0 {
        return (
            Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            true,
        );
    }
    if index == 1 {
        return (
            Color::Rgb {
                r: 0,
                g: 255,
                b: 65,
            },
            false,
        );
    }
    if index == 2 {
        return (
            Color::Rgb {
                r: 40,
                g: 235,
                b: 95,
            },
            false,
        );
    }

    let t = (index as f32) / (trail_length.max(2) as f32 - 1.0);
    let (r, g, b) = if t < 0.45 {
        // medium green -> dark green
        lerp_rgb((0, 175, 65), (0, 110, 45), t / 0.45)
    } else if t < 0.80 {
        // dark green -> very dark green
        lerp_rgb((0, 110, 45), (0, 45, 22), (t - 0.45) / 0.35)
    } else {
        // very dark green -> almost black-green
        lerp_rgb((0, 45, 22), (0, 8, 4), (t - 0.80) / 0.20)
    };

    (Color::Rgb { r, g, b }, false)
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let clamped = t.clamp(0.0, 1.0);
    let r = a.0 as f32 + (b.0 as f32 - a.0 as f32) * clamped;
    let g = a.1 as f32 + (b.1 as f32 - a.1 as f32) * clamped;
    let b2 = a.2 as f32 + (b.2 as f32 - a.2 as f32) * clamped;
    (r as u8, g as u8, b2 as u8)
}

fn apply_pulse(color: Color, x: u16, y: u16, frame: u64, depth: usize) -> Color {
    let Color::Rgb { r, g, b } = color else {
        return color;
    };

    // Small traveling shimmer wave; strongest near the head.
    let wave = ((frame + x as u64 * 3 + y as u64 * 5) % 24) as f32 / 24.0;
    let tri = if wave < 0.5 {
        wave * 2.0
    } else {
        (1.0 - wave) * 2.0
    };
    let strength = (1.0 - (depth as f32 * 0.04)).clamp(0.45, 1.0);
    let factor = 0.90 + (tri * 0.18 * strength);

    Color::Rgb {
        r: (r as f32 * factor).clamp(0.0, 255.0) as u8,
        g: (g as f32 * factor).clamp(0.0, 255.0) as u8,
        b: (b as f32 * factor).clamp(0.0, 255.0) as u8,
    }
}
