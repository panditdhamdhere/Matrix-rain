use std::collections::{HashMap, HashSet};
use std::io::{Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};

use crate::column::Column;
use crate::config::Theme;

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

pub struct OverlayState {
    pub visible: bool,
    pub paused: bool,
    pub fps: f32,
    pub theme: Theme,
    pub density: f32,
    pub speed_scale: f32,
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

    pub fn render(
        &mut self,
        stdout: &mut Stdout,
        columns: &[Column],
        overlay: &OverlayState,
    ) -> std::io::Result<()> {
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
                let (base_color, bold) = color_for_index(i, column.trail_length, overlay.theme);
                let color = apply_pulse(base_color, column.x, y as u16, self.frame, i);
                current.insert((column.x, y as u16), CellStyle { glyph, color, bold });
            }
        }
        if overlay.visible {
            add_overlay_line(self.width, &mut current, overlay);
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

fn color_for_index(index: usize, trail_length: usize, theme: Theme) -> (Color, bool) {
    let palette = palette_for(theme);
    if index == 0 {
        return (rgb(palette.head), true);
    }
    if index == 1 {
        return (rgb(palette.second), false);
    }
    if index == 2 {
        return (rgb(palette.third), false);
    }

    let t = (index as f32) / (trail_length.max(2) as f32 - 1.0);
    let (r, g, b) = if t < 0.45 {
        lerp_rgb(palette.mid, palette.dark, t / 0.45)
    } else if t < 0.80 {
        lerp_rgb(palette.dark, palette.vdark, (t - 0.45) / 0.35)
    } else {
        lerp_rgb(palette.vdark, palette.near_black, (t - 0.80) / 0.20)
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

#[derive(Clone, Copy)]
struct ThemePalette {
    head: (u8, u8, u8),
    second: (u8, u8, u8),
    third: (u8, u8, u8),
    mid: (u8, u8, u8),
    dark: (u8, u8, u8),
    vdark: (u8, u8, u8),
    near_black: (u8, u8, u8),
}

fn palette_for(theme: Theme) -> ThemePalette {
    match theme {
        Theme::Matrix => ThemePalette {
            head: (255, 255, 255),
            second: (0, 255, 65),
            third: (40, 235, 95),
            mid: (0, 175, 65),
            dark: (0, 110, 45),
            vdark: (0, 45, 22),
            near_black: (0, 8, 4),
        },
        Theme::Amber => ThemePalette {
            head: (255, 255, 255),
            second: (255, 200, 30),
            third: (245, 165, 25),
            mid: (210, 120, 18),
            dark: (140, 70, 12),
            vdark: (70, 30, 6),
            near_black: (12, 5, 2),
        },
        Theme::Ice => ThemePalette {
            head: (255, 255, 255),
            second: (130, 245, 255),
            third: (90, 225, 255),
            mid: (70, 185, 230),
            dark: (35, 115, 165),
            vdark: (18, 58, 95),
            near_black: (4, 12, 22),
        },
    }
}

fn rgb(rgb: (u8, u8, u8)) -> Color {
    Color::Rgb {
        r: rgb.0,
        g: rgb.1,
        b: rgb.2,
    }
}

fn add_overlay_line(
    width: u16,
    current: &mut HashMap<(u16, u16), CellStyle>,
    overlay: &OverlayState,
) {
    if width == 0 {
        return;
    }
    let status = if overlay.paused { "PAUSED" } else { "RUNNING" };
    let text = format!(
        "[{}] {} | fps:{:>5.1} | density:{:.2} | speed:{:.2} | q quit | space pause | h hide",
        status,
        overlay.theme.as_str(),
        overlay.fps,
        overlay.density,
        overlay.speed_scale
    );
    for (idx, ch) in text.chars().take(width as usize).enumerate() {
        current.insert(
            (idx as u16, 0),
            CellStyle {
                glyph: ch,
                color: Color::Rgb {
                    r: 170,
                    g: 170,
                    b: 170,
                },
                bold: false,
            },
        );
    }
}
