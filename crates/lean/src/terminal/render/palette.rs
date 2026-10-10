use alacritty_terminal::term::color::{COUNT, Colors};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
use gpui_kit::component::Theme;
use gpui_kit::{Hsla, Rgba, rgb};

pub(crate) const DIM_OPACITY: f32 = 0.7;

const BLACK: u32 = 0x2a2a2f;
const BRIGHT_WHITE: u32 = 0xf4f4f2;
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
const CUBE_START: usize = 16;
const GRAY_START: usize = 232;
const NAMED_START: usize = 256;
const FOREGROUND: usize = NamedColor::Foreground as usize;
const BACKGROUND: usize = NamedColor::Background as usize;
const CURSOR: usize = NamedColor::Cursor as usize;
const DIM_BLACK: usize = NamedColor::DimBlack as usize;
const BRIGHT_FOREGROUND: usize = NamedColor::BrightForeground as usize;

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    ansi: [Hsla; 16],
    foreground: Hsla,
    background: Hsla,
    cursor: Hsla,
    selection: Hsla,
}

impl Palette {
    pub(crate) fn new(theme: &Theme) -> Self {
        Self {
            ansi: [
                rgb(BLACK).into(),
                theme.red,
                theme.green,
                theme.yellow,
                theme.blue,
                theme.magenta,
                theme.cyan,
                theme.foreground,
                theme.muted_foreground,
                theme.red_light,
                theme.green_light,
                theme.yellow_light,
                theme.blue_light,
                theme.magenta_light,
                theme.cyan_light,
                rgb(BRIGHT_WHITE).into(),
            ],
            foreground: theme.foreground,
            background: theme.sidebar,
            cursor: theme.caret,
            selection: theme.selection,
        }
    }

    pub(crate) fn background(&self) -> Hsla {
        self.background
    }

    pub(crate) fn cursor(&self) -> Hsla {
        self.cursor
    }

    pub(crate) fn selection(&self) -> Hsla {
        self.selection
    }

    pub(crate) fn resolve(&self, color: Color, overrides: &Colors) -> Hsla {
        match color {
            Color::Spec(spec) => hsla_from_rgb(spec),
            Color::Named(named) => self.indexed(named as usize, overrides),
            Color::Indexed(index) => self.indexed(usize::from(index), overrides),
        }
    }

    pub(crate) fn indexed(&self, index: usize, overrides: &Colors) -> Hsla {
        if index < COUNT
            && let Some(color) = overrides[index]
        {
            return hsla_from_rgb(color);
        }
        self.default_color(index)
    }

    fn default_color(&self, index: usize) -> Hsla {
        match index {
            0..CUBE_START => self.ansi[index],
            CUBE_START..GRAY_START => cube_color(index - CUBE_START),
            GRAY_START..NAMED_START => gray_color(index - GRAY_START),
            FOREGROUND | BRIGHT_FOREGROUND => self.foreground,
            BACKGROUND => self.background,
            CURSOR => self.cursor,
            DIM_BLACK..BRIGHT_FOREGROUND => self.ansi[index - DIM_BLACK].opacity(DIM_OPACITY),
            _ => self.foreground.opacity(DIM_OPACITY),
        }
    }
}

pub(crate) fn rgb_from_hsla(color: Hsla) -> Rgb {
    let rgba = color.to_rgb();
    Rgb {
        r: channel_byte(rgba.r),
        g: channel_byte(rgba.g),
        b: channel_byte(rgba.b),
    }
}

fn cube_color(offset: usize) -> Hsla {
    hsla_from_rgb(Rgb {
        r: CUBE_LEVELS[offset / 36 % 6],
        g: CUBE_LEVELS[offset / 6 % 6],
        b: CUBE_LEVELS[offset % 6],
    })
}

fn gray_color(offset: usize) -> Hsla {
    let level = u8::try_from(8 + offset * 10).unwrap_or(u8::MAX);
    hsla_from_rgb(Rgb {
        r: level,
        g: level,
        b: level,
    })
}

fn hsla_from_rgb(color: Rgb) -> Hsla {
    Rgba {
        r: channel_fraction(color.r),
        g: channel_fraction(color.g),
        b: channel_fraction(color.b),
        a: 1.,
    }
    .into()
}

fn channel_fraction(value: u8) -> f32 {
    f32::from(value) / 255.
}

fn channel_byte(value: f32) -> u8 {
    (value.clamp(0., 1.) * 255.).round() as u8
}
