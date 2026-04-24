use std::env;
use std::fmt::{Display, Formatter};

use crate::column::{ColumnSettings, GlyphStyle};

#[derive(Clone, Copy)]
pub enum Theme {
    Matrix,
    Amber,
    Ice,
}

impl Theme {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "matrix" => Some(Self::Matrix),
            "amber" => Some(Self::Amber),
            "ice" => Some(Self::Ice),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Matrix => "matrix",
            Self::Amber => "amber",
            Self::Ice => "ice",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppConfig {
    pub fps: u64,
    pub density: f32,
    pub speed_scale: f32,
    pub glyph_style: GlyphStyle,
    pub theme: Theme,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            density: 1.0,
            speed_scale: 1.0,
            glyph_style: GlyphStyle::Balanced,
            theme: Theme::Matrix,
        }
    }
}

impl AppConfig {
    pub fn frame_time(self) -> std::time::Duration {
        std::time::Duration::from_nanos(1_000_000_000 / self.fps.max(1))
    }

    pub fn column_settings(self) -> ColumnSettings {
        ColumnSettings {
            glyph_style: self.glyph_style,
            speed_scale: self.speed_scale,
            ..ColumnSettings::default()
        }
    }

    pub fn from_args() -> Result<Self, ConfigError> {
        let mut cfg = Self::default();

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => return Err(ConfigError::Help),
                "--fps" => {
                    let value = next_value(&mut args, "--fps")?;
                    cfg.fps = parse_fps(&value)?;
                }
                "--density" => {
                    let value = next_value(&mut args, "--density")?;
                    cfg.density = parse_density(&value)?;
                }
                "--speed" => {
                    let value = next_value(&mut args, "--speed")?;
                    cfg.speed_scale = parse_speed_scale(&value)?;
                }
                "--glyph-style" => {
                    let value = next_value(&mut args, "--glyph-style")?;
                    cfg.glyph_style = GlyphStyle::from_str(&value).ok_or({
                        ConfigError::InvalidArgValue {
                            key: "--glyph-style",
                            value,
                            expected: "classic|balanced|ascii",
                        }
                    })?;
                }
                "--theme" => {
                    let value = next_value(&mut args, "--theme")?;
                    cfg.theme = Theme::from_str(&value).ok_or(ConfigError::InvalidArgValue {
                        key: "--theme",
                        value,
                        expected: "matrix|amber|ice",
                    })?;
                }
                _ => {
                    return Err(ConfigError::UnknownArg(arg));
                }
            }
        }

        Ok(cfg)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Help,
    MissingArgValue(&'static str),
    InvalidArgValue {
        key: &'static str,
        value: String,
        expected: &'static str,
    },
    UnknownArg(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Help => write!(f, "{}", usage_text()),
            ConfigError::MissingArgValue(key) => {
                write!(f, "missing value for {key}\n{}", usage_text())
            }
            ConfigError::InvalidArgValue {
                key,
                value,
                expected,
            } => write!(
                f,
                "invalid value '{value}' for {key}; expected {expected}\n{}",
                usage_text()
            ),
            ConfigError::UnknownArg(arg) => write!(f, "unknown argument '{arg}'\n{}", usage_text()),
        }
    }
}

fn parse_fps(value: &str) -> Result<u64, ConfigError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidArgValue {
            key: "--fps",
            value: value.to_owned(),
            expected: "integer in range 10..=240",
        })?;
    if (10..=240).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(ConfigError::InvalidArgValue {
            key: "--fps",
            value: value.to_owned(),
            expected: "integer in range 10..=240",
        })
    }
}

fn parse_density(value: &str) -> Result<f32, ConfigError> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| ConfigError::InvalidArgValue {
            key: "--density",
            value: value.to_owned(),
            expected: "number in range 0.1..=1.0",
        })?;
    if (0.1..=1.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(ConfigError::InvalidArgValue {
            key: "--density",
            value: value.to_owned(),
            expected: "number in range 0.1..=1.0",
        })
    }
}

fn parse_speed_scale(value: &str) -> Result<f32, ConfigError> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| ConfigError::InvalidArgValue {
            key: "--speed",
            value: value.to_owned(),
            expected: "number in range 0.5..=2.0",
        })?;
    if (0.5..=2.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(ConfigError::InvalidArgValue {
            key: "--speed",
            value: value.to_owned(),
            expected: "number in range 0.5..=2.0",
        })
    }
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    key: &'static str,
) -> Result<String, ConfigError> {
    args.next().ok_or(ConfigError::MissingArgValue(key))
}

pub fn usage_text() -> &'static str {
    "Usage: matrix-rain [options]
  --fps <10-240>              Target frame rate (default: 30)
  --density <0.1-1.0>         Active column density (default: 1.0)
  --speed <0.5-2.0>           Global speed multiplier (default: 1.0)
  --glyph-style <classic|balanced|ascii>
                              Character style (default: balanced)
  --theme <matrix|amber|ice>  Color theme (default: matrix)
  -h, --help                  Show this help"
}
