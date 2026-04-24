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
#[allow(dead_code)]
enum GlyphStyle {
    ClassicMatrix,
    Balanced,
    AsciiGlitch,
}

const GLYPH_STYLE: GlyphStyle = GlyphStyle::Balanced;

#[derive(Clone)]
pub struct Column {
    pub x: u16,
    pub y: f32,
    pub speed: f32,
    pub trail: Vec<char>,
    pub trail_length: usize,
    pub active: bool,
    pub delay: u32,
}

impl Column {
    pub fn new(x: u16, rng: &mut impl Rng) -> Self {
        let trail_length = rng.gen_range(MIN_TRAIL..=MAX_TRAIL);
        let mut column = Self {
            x,
            y: -(trail_length as f32),
            speed: random_speed_tier(rng),
            trail: vec![' '; trail_length],
            trail_length,
            active: false,
            delay: rng.gen_range(0..72),
        };
        column.randomize_trail(rng);
        column
    }

    pub fn reset_fall(&mut self, rng: &mut impl Rng) {
        self.trail_length = rng.gen_range(MIN_TRAIL..=MAX_TRAIL);
        self.trail.resize(self.trail_length, ' ');
        self.randomize_trail(rng);
        self.y = -(self.trail_length as f32);
        self.speed = random_speed_tier(rng);
        self.active = false;
        self.delay = rng.gen_range(4..96);
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
            *ch = random_char(rng);
        }
    }

    fn mutate(&mut self, rng: &mut impl Rng) {
        if !self.trail.is_empty() {
            self.trail[0] = random_char(rng);
        }
        for ch in &mut self.trail {
            if rng.gen_bool(0.14) {
                *ch = random_char(rng);
            }
        }
    }
}

fn random_speed_tier(rng: &mut impl Rng) -> f32 {
    match rng.gen_range(0..3) {
        0 => rng.gen_range(0.35..=0.60), // slow
        1 => rng.gen_range(0.75..=1.10), // medium
        _ => rng.gen_range(1.20..=1.50), // fast
    }
}

fn random_char(rng: &mut impl Rng) -> char {
    match GLYPH_STYLE {
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
