use rand::Rng;

const SYMBOLS: &[char] = &['!', '@', '#', '$', '%', '^', '&', '*'];
const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const LATIN: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];
const KATAKANA: &[char] = &[
    // Half-width katakana keeps the Matrix vibe but reads less like normal words.
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ',
    'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
];

const MIN_TRAIL: usize = 8;
const MAX_TRAIL: usize = 22;

#[derive(Clone, Copy)]
pub enum GlyphStyle {
    ClassicMatrix,
    Balanced,
    AsciiGlitch,
}

impl GlyphStyle {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "classic" => Some(Self::ClassicMatrix),
            "balanced" => Some(Self::Balanced),
            "ascii" => Some(Self::AsciiGlitch),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColumnSettings {
    pub glyph_style: GlyphStyle,
    pub speed_scale: f32,
    pub min_trail: usize,
    pub max_trail: usize,
    pub initial_delay_max: u32,
    pub restart_delay_min: u32,
    pub restart_delay_max: u32,
    pub mutation_chance: f64,
}

impl Default for ColumnSettings {
    fn default() -> Self {
        Self {
            glyph_style: GlyphStyle::Balanced,
            speed_scale: 1.0,
            min_trail: MIN_TRAIL,
            max_trail: MAX_TRAIL,
            initial_delay_max: 72,
            restart_delay_min: 4,
            restart_delay_max: 96,
            mutation_chance: 0.14,
        }
    }
}

#[derive(Clone)]
pub struct Column {
    pub x: u16,
    pub y: f32,
    pub speed: f32,
    pub trail: Vec<char>,
    pub trail_length: usize,
    pub active: bool,
    pub delay: u32,
    settings: ColumnSettings,
}

impl Column {
    pub fn new(x: u16, settings: ColumnSettings, rng: &mut impl Rng) -> Self {
        let trail_length = rng.gen_range(settings.min_trail..=settings.max_trail);
        let mut column = Self {
            x,
            y: -(trail_length as f32),
            speed: random_speed_tier(rng, settings.speed_scale),
            trail: vec![' '; trail_length],
            trail_length,
            active: false,
            delay: rng.gen_range(0..settings.initial_delay_max.max(1)),
            settings,
        };
        column.randomize_trail(rng);
        column
    }

    pub fn reset_fall(&mut self, rng: &mut impl Rng) {
        self.trail_length = rng.gen_range(self.settings.min_trail..=self.settings.max_trail);
        self.trail.resize(self.trail_length, ' ');
        self.randomize_trail(rng);
        self.y = -(self.trail_length as f32);
        self.speed = random_speed_tier(rng, self.settings.speed_scale);
        self.active = false;
        let restart_max = self
            .settings
            .restart_delay_max
            .max(self.settings.restart_delay_min + 1);
        self.delay = rng.gen_range(self.settings.restart_delay_min..restart_max);
    }

    pub fn tick(&mut self, terminal_height: u16, rng: &mut impl Rng) {
        if !self.active {
            if self.delay > 0 {
                self.delay -= 1;
                return;
            }
            self.active = true;
            self.y = -(self.trail_length as f32);
        }

        self.y += self.speed;
        self.mutate(rng);

        if self.y - self.trail_length as f32 > terminal_height as f32 {
            self.reset_fall(rng);
        }
    }

    fn randomize_trail(&mut self, rng: &mut impl Rng) {
        for ch in &mut self.trail {
            *ch = random_char(self.settings.glyph_style, rng);
        }
    }

    fn mutate(&mut self, rng: &mut impl Rng) {
        if !self.trail.is_empty() {
            self.trail[0] = random_char(self.settings.glyph_style, rng);
        }
        for ch in &mut self.trail {
            if rng.gen_bool(self.settings.mutation_chance) {
                *ch = random_char(self.settings.glyph_style, rng);
            }
        }
    }
}

fn random_speed_tier(rng: &mut impl Rng, speed_scale: f32) -> f32 {
    let base = match rng.gen_range(0..3) {
        0 => rng.gen_range(0.35..=0.60), // slow
        1 => rng.gen_range(0.75..=1.10), // medium
        _ => rng.gen_range(1.20..=1.50), // fast
    };
    base * speed_scale
}

fn random_char(glyph_style: GlyphStyle, rng: &mut impl Rng) -> char {
    match glyph_style {
        GlyphStyle::ClassicMatrix => match rng.gen_range(0..10) {
            0..=6 => KATAKANA[rng.gen_range(0..KATAKANA.len())],
            7..=8 => DIGITS[rng.gen_range(0..DIGITS.len())],
            _ => SYMBOLS[rng.gen_range(0..SYMBOLS.len())],
        },
        GlyphStyle::Balanced => match rng.gen_range(0..10) {
            0..=3 => KATAKANA[rng.gen_range(0..KATAKANA.len())],
            4..=6 => DIGITS[rng.gen_range(0..DIGITS.len())],
            7..=8 => LATIN[rng.gen_range(0..LATIN.len())],
            _ => SYMBOLS[rng.gen_range(0..SYMBOLS.len())],
        },
        GlyphStyle::AsciiGlitch => match rng.gen_range(0..10) {
            0..=4 => DIGITS[rng.gen_range(0..DIGITS.len())],
            5..=8 => LATIN[rng.gen_range(0..LATIN.len())],
            _ => SYMBOLS[rng.gen_range(0..SYMBOLS.len())],
        },
    }
}
