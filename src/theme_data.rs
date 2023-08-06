use iced_core::Color;
use palette::rgb::Rgb;
use palette::{DarkenAssign, FromColor, LightenAssign, Mix, Okhsl, Srgb};

const DEFAULT_THEME_NAME: &str = "Ferra";

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: Colors,
}

impl Theme {
    pub fn new(name: String, palette: &Palette) -> Self {
        Theme {
            name,
            colors: Colors::new(palette),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: DEFAULT_THEME_NAME.to_string(),
            colors: Colors::new(&Palette::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub background: Subpalette,
    pub text: Subpalette,
    pub action: Subpalette,
    pub accent: Subpalette,
    pub alert: Subpalette,
    pub error: Subpalette,
    pub info: Subpalette,
    pub success: Subpalette,
}

impl Colors {
    pub fn new(palette: &Palette) -> Self {
        Colors {
            background: Subpalette::from_color(palette.background, palette),
            text: Subpalette::from_color(palette.text, palette),
            action: Subpalette::from_color(palette.action, palette),
            accent: Subpalette::from_color(palette.accent, palette),
            alert: Subpalette::from_color(palette.alert, palette),
            error: Subpalette::from_color(palette.error, palette),
            info: Subpalette::from_color(palette.info, palette),
            success: Subpalette::from_color(palette.success, palette),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subpalette {
    pub base: Color,
    pub light: Color,
    pub lighter: Color,
    pub lightest: Color,
    pub dark: Color,
    pub darker: Color,
    pub darkest: Color,
    pub low_alpha: Color,
    pub med_alpha: Color,
    pub high_alpha: Color,
}

impl Subpalette {
    pub fn from_color(color: Color, palette: &Palette) -> Subpalette {
        let is_dark = is_dark(palette.background);

        Subpalette {
            base: color,
            light: lighten(color, 0.03),
            lighter: lighten(color, 0.06),
            lightest: lighten(color, 0.12),
            dark: darken(color, 0.03),
            darker: darken(color, 0.06),
            darkest: darken(color, 0.12),
            low_alpha: if is_dark {
                alpha(color, 0.4)
            } else {
                alpha(color, 0.8)
            },
            med_alpha: if is_dark {
                alpha(color, 0.2)
            } else {
                alpha(color, 0.4)
            },
            high_alpha: if is_dark {
                alpha(color, 0.1)
            } else {
                alpha(color, 0.3)
            },
        }
    }

    pub fn is_dark(&self) -> bool {
        is_dark(self.base)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub text: Color,
    pub action: Color,
    pub accent: Color,
    pub alert: Color,
    pub error: Color,
    pub info: Color,
    pub success: Color,
}

impl Default for Palette {
    fn default() -> Palette {
        Palette {
            background: hex_to_color("#2b292d").unwrap(),
            text: hex_to_color("#fecdb2").unwrap(),
            action: hex_to_color("#b1b695").unwrap(),
            accent: hex_to_color("#d1d1e0").unwrap(),
            alert: hex_to_color("#ffa07a").unwrap(),
            error: hex_to_color("#e06b75").unwrap(),
            info: hex_to_color("#f5d76e").unwrap(),
            success: hex_to_color("#b1b695").unwrap(),
        }
    }
}

fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() == 7 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);

        return match (hash, r, g, b) {
            ("#", Ok(r), Ok(g), Ok(b)) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }),
            _ => None,
        };
    }

    None
}

pub fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.5
}

pub fn to_hsl(color: Color) -> Okhsl {
    let mut hsl = Okhsl::from_color(Rgb::from(color));
    if hsl.saturation.is_nan() {
        hsl.saturation = Okhsl::max_saturation();
    }

    hsl
}

pub fn from_hsl(hsl: Okhsl) -> Color {
    Srgb::from_color(hsl).into()
}

pub fn alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let a_hsl = to_hsl(a);
    let b_hsl = to_hsl(b);

    let mixed = a_hsl.mix(b_hsl, factor);
    from_hsl(mixed)
}

pub fn lighten(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lighten_fixed_assign(amount);

    from_hsl(hsl)
}

pub fn darken(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.darken_fixed_assign(amount);

    from_hsl(hsl)
}
