//! LESS 的值类型和实用工具
//!
//! 此模块定义了可在 LESS 表达式中使用的值类型，
//! 包括单位、颜色和值操作的实用函数。

use super::Position;
use std::fmt;

/// CSS 单位类型
#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    // 长度单位
    Px,
    Em,
    Rem,
    Ex,
    Ch,
    Vw,
    Vh,
    Vmin,
    Vmax,
    Cm,
    Mm,
    Q,
    In,
    Pt,
    Pc,

    // 角度单位
    Deg,
    Grad,
    Rad,
    Turn,

    // Time units
    S,
    Ms,

    // Frequency units
    Hz,
    Khz,

    // Resolution units
    Dpi,
    Dpcm,
    Dppx,

    // Percentage
    Percent,

    // Dimensionless
    None,
}

/// Color representation with various formats
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64,
    pub position: Position,
}

/// Named colors commonly used in CSS
#[derive(Debug, Clone, PartialEq)]
pub enum NamedColor {
    Transparent,
    AliceBlue,
    AntiqueWhite,
    Aqua,
    Aquamarine,
    Azure,
    Beige,
    Bisque,
    Black,
    BlanchedAlmond,
    Blue,
    BlueViolet,
    Brown,
    BurlyWood,
    CadetBlue,
    Chartreuse,
    Chocolate,
    Coral,
    CornflowerBlue,
    Cornsilk,
    Crimson,
    Cyan,
    DarkBlue,
    DarkCyan,
    DarkGoldenRod,
    DarkGray,
    DarkGreen,
    DarkKhaki,
    DarkMagenta,
    DarkOliveGreen,
    DarkOrange,
    DarkOrchid,
    DarkRed,
    DarkSalmon,
    DarkSeaGreen,
    DarkSlateBlue,
    DarkSlateGray,
    DarkTurquoise,
    DarkViolet,
    DeepPink,
    DeepSkyBlue,
    DimGray,
    DodgerBlue,
    FireBrick,
    FloralWhite,
    ForestGreen,
    Fuchsia,
    Gainsboro,
    GhostWhite,
    Gold,
    GoldenRod,
    Gray,
    Green,
    GreenYellow,
    HoneyDew,
    HotPink,
    IndianRed,
    Indigo,
    Ivory,
    Khaki,
    Lavender,
    LavenderBlush,
    LawnGreen,
    LemonChiffon,
    LightBlue,
    LightCoral,
    LightCyan,
    LightGoldenRodYellow,
    LightGray,
    LightGreen,
    LightPink,
    LightSalmon,
    LightSeaGreen,
    LightSkyBlue,
    LightSlateGray,
    LightSteelBlue,
    LightYellow,
    Lime,
    LimeGreen,
    Linen,
    Magenta,
    Maroon,
    MediumAquaMarine,
    MediumBlue,
    MediumOrchid,
    MediumPurple,
    MediumSeaGreen,
    MediumSlateBlue,
    MediumSpringGreen,
    MediumTurquoise,
    MediumVioletRed,
    MidnightBlue,
    MintCream,
    MistyRose,
    Moccasin,
    NavajoWhite,
    Navy,
    OldLace,
    Olive,
    OliveDrab,
    Orange,
    OrangeRed,
    Orchid,
    PaleGoldenRod,
    PaleGreen,
    PaleTurquoise,
    PaleVioletRed,
    PapayaWhip,
    PeachPuff,
    Peru,
    Pink,
    Plum,
    PowderBlue,
    Purple,
    RebeccaPurple,
    Red,
    RosyBrown,
    RoyalBlue,
    SaddleBrown,
    Salmon,
    SandyBrown,
    SeaGreen,
    SeaShell,
    Sienna,
    Silver,
    SkyBlue,
    SlateBlue,
    SlateGray,
    Snow,
    SpringGreen,
    SteelBlue,
    Tan,
    Teal,
    Thistle,
    Tomato,
    Turquoise,
    Violet,
    Wheat,
    White,
    WhiteSmoke,
    Yellow,
    YellowGreen,
}

/// Numeric value with optional unit
#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: f64,
    pub unit: Unit,
    pub position: Position,
}

/// String value types
#[derive(Debug, Clone, PartialEq)]
pub enum StringType {
    /// Quoted string: "hello" or 'hello'
    Quoted(String),
    /// Unquoted identifier: hello
    Unquoted(String),
    /// URL: url(path)
    Url(String),
}

impl Unit {
    /// Parse a unit from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Length units
            "px" => Some(Unit::Px),
            "em" => Some(Unit::Em),
            "rem" => Some(Unit::Rem),
            "ex" => Some(Unit::Ex),
            "ch" => Some(Unit::Ch),
            "vw" => Some(Unit::Vw),
            "vh" => Some(Unit::Vh),
            "vmin" => Some(Unit::Vmin),
            "vmax" => Some(Unit::Vmax),
            "cm" => Some(Unit::Cm),
            "mm" => Some(Unit::Mm),
            "q" => Some(Unit::Q),
            "in" => Some(Unit::In),
            "pt" => Some(Unit::Pt),
            "pc" => Some(Unit::Pc),

            // Angle units
            "deg" => Some(Unit::Deg),
            "grad" => Some(Unit::Grad),
            "rad" => Some(Unit::Rad),
            "turn" => Some(Unit::Turn),

            // Time units
            "s" => Some(Unit::S),
            "ms" => Some(Unit::Ms),

            // Frequency units
            "hz" => Some(Unit::Hz),
            "khz" => Some(Unit::Khz),

            // Resolution units
            "dpi" => Some(Unit::Dpi),
            "dpcm" => Some(Unit::Dpcm),
            "dppx" => Some(Unit::Dppx),

            // Percentage
            "%" => Some(Unit::Percent),

            _ => None,
        }
    }

    /// Convert unit to string
    pub fn to_string(&self) -> &'static str {
        match self {
            // Length units
            Unit::Px => "px",
            Unit::Em => "em",
            Unit::Rem => "rem",
            Unit::Ex => "ex",
            Unit::Ch => "ch",
            Unit::Vw => "vw",
            Unit::Vh => "vh",
            Unit::Vmin => "vmin",
            Unit::Vmax => "vmax",
            Unit::Cm => "cm",
            Unit::Mm => "mm",
            Unit::Q => "q",
            Unit::In => "in",
            Unit::Pt => "pt",
            Unit::Pc => "pc",

            // Angle units
            Unit::Deg => "deg",
            Unit::Grad => "grad",
            Unit::Rad => "rad",
            Unit::Turn => "turn",

            // Time units
            Unit::S => "s",
            Unit::Ms => "ms",

            // Frequency units
            Unit::Hz => "Hz",
            Unit::Khz => "kHz",

            // Resolution units
            Unit::Dpi => "dpi",
            Unit::Dpcm => "dpcm",
            Unit::Dppx => "dppx",

            // Percentage
            Unit::Percent => "%",

            // Dimensionless
            Unit::None => "",
        }
    }

    /// Check if this is a length unit
    pub fn is_length(&self) -> bool {
        matches!(
            self,
            Unit::Px
                | Unit::Em
                | Unit::Rem
                | Unit::Ex
                | Unit::Ch
                | Unit::Vw
                | Unit::Vh
                | Unit::Vmin
                | Unit::Vmax
                | Unit::Cm
                | Unit::Mm
                | Unit::Q
                | Unit::In
                | Unit::Pt
                | Unit::Pc
        )
    }

    /// Check if this is an angle unit
    pub fn is_angle(&self) -> bool {
        matches!(self, Unit::Deg | Unit::Grad | Unit::Rad | Unit::Turn)
    }

    /// Check if this is a time unit
    pub fn is_time(&self) -> bool {
        matches!(self, Unit::S | Unit::Ms)
    }

    /// Check if this is a frequency unit
    pub fn is_frequency(&self) -> bool {
        matches!(self, Unit::Hz | Unit::Khz)
    }

    /// Check if this is a resolution unit
    pub fn is_resolution(&self) -> bool {
        matches!(self, Unit::Dpi | Unit::Dpcm | Unit::Dppx)
    }

    /// Check if units are compatible for arithmetic operations
    pub fn is_compatible(&self, other: &Unit) -> bool {
        match (self, other) {
            (Unit::None, _) | (_, Unit::None) => true,
            (Unit::Percent, _) | (_, Unit::Percent) => true,
            (a, b) if a == b => true,
            (a, b) if a.is_length() && b.is_length() => true,
            (a, b) if a.is_angle() && b.is_angle() => true,
            (a, b) if a.is_time() && b.is_time() => true,
            (a, b) if a.is_frequency() && b.is_frequency() => true,
            (a, b) if a.is_resolution() && b.is_resolution() => true,
            _ => false,
        }
    }
}

impl Color {
    /// Create a new color from RGB values
    pub fn rgb(red: u8, green: u8, blue: u8, position: Position) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
            position,
        }
    }

    /// Create a new color from RGBA values
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: f64, position: Position) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: alpha.clamp(0.0, 1.0),
            position,
        }
    }

    /// Create a color from HSL values
    pub fn hsl(hue: f64, saturation: f64, lightness: f64, position: Position) -> Self {
        let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
        Self::rgb(r, g, b, position)
    }

    /// Create a color from HSLA values
    pub fn hsla(hue: f64, saturation: f64, lightness: f64, alpha: f64, position: Position) -> Self {
        let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
        Self::rgba(r, g, b, alpha, position)
    }

    /// Parse a hex color string
    pub fn from_hex(hex: &str, position: Position) -> Result<Self, &'static str> {
        let hex = hex.trim_start_matches('#');

        let (red, green, blue, alpha) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                (r, g, b, 1.0)
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")? as f64
                    / 255.0;
                (r, g, b, a)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                (r, g, b, 1.0)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")? as f64
                    / 255.0;
                (r, g, b, a)
            }
            _ => return Err("Invalid hex color length"),
        };

        Ok(Self {
            red,
            green,
            blue,
            alpha,
            position,
        })
    }

    /// Convert to HSL values
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        rgb_to_hsl(self.red, self.green, self.blue)
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.alpha < 1.0 {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.red,
                self.green,
                self.blue,
                (self.alpha * 255.0) as u8
            )
        } else {
            // Check if we can use short form (e.g., #333 instead of #333333)
            if self.red % 17 == 0 && self.green % 17 == 0 && self.blue % 17 == 0 {
                format!(
                    "#{:x}{:x}{:x}",
                    self.red / 17,
                    self.green / 17,
                    self.blue / 17
                )
            } else {
                format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
            }
        }
    }

    /// Convert to CSS string representation
    pub fn to_css(&self) -> String {
        if self.alpha < 1.0 {
            format!(
                "rgba({}, {}, {}, {})",
                self.red, self.green, self.blue, self.alpha
            )
        } else {
            // Use hex format for opaque colors
            self.to_hex()
        }
    }

    /// Lighten the color by a percentage
    pub fn lighten(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_l = (l + amount / 100.0).clamp(0.0, 1.0);
        Self::hsla(h, s, new_l, self.alpha, self.position.clone())
    }

    /// Darken the color by a percentage
    pub fn darken(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_l = (l - amount / 100.0).clamp(0.0, 1.0);
        Self::hsla(h, s, new_l, self.alpha, self.position.clone())
    }

    /// Saturate the color by a percentage
    pub fn saturate(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s + amount / 100.0).clamp(0.0, 1.0);
        Self::hsla(h, new_s, l, self.alpha, self.position.clone())
    }

    /// Desaturate the color by a percentage
    pub fn desaturate(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s - amount / 100.0).clamp(0.0, 1.0);
        Self::hsla(h, new_s, l, self.alpha, self.position.clone())
    }

    /// Fade the color by a percentage
    pub fn fade(&self, amount: f64) -> Self {
        let new_alpha = (amount / 100.0).clamp(0.0, 1.0);
        Self::rgba(
            self.red,
            self.green,
            self.blue,
            new_alpha,
            self.position.clone(),
        )
    }

    /// Mix two colors
    pub fn mix(&self, other: &Color, weight: f64) -> Self {
        let w = weight.clamp(0.0, 1.0);
        let w2 = 1.0 - w;

        let red = (self.red as f64 * w + other.red as f64 * w2) as u8;
        let green = (self.green as f64 * w + other.green as f64 * w2) as u8;
        let blue = (self.blue as f64 * w + other.blue as f64 * w2) as u8;
        let alpha = self.alpha * w + other.alpha * w2;

        Self::rgba(red, green, blue, alpha, self.position.clone())
    }
}

impl Number {
    /// Create a new number with no unit
    pub fn new(value: f64, position: Position) -> Self {
        Self {
            value,
            unit: Unit::None,
            position,
        }
    }

    /// Create a new number with a unit
    pub fn with_unit(value: f64, unit: Unit, position: Position) -> Self {
        Self {
            value,
            unit,
            position,
        }
    }

    /// Convert to CSS string
    pub fn to_css(&self) -> String {
        if self.unit == Unit::None {
            format!("{}", self.value)
        } else {
            format!("{}{}", self.value, self.unit.to_string())
        }
    }

    /// Add two numbers
    pub fn add(&self, other: &Number) -> Result<Number, &'static str> {
        if !self.unit.is_compatible(&other.unit) {
            return Err("Incompatible units for addition");
        }

        let result_unit = if self.unit == Unit::None {
            other.unit.clone()
        } else {
            self.unit.clone()
        };

        Ok(Number::with_unit(
            self.value + other.value,
            result_unit,
            self.position.clone(),
        ))
    }

    /// Subtract two numbers
    pub fn subtract(&self, other: &Number) -> Result<Number, &'static str> {
        if !self.unit.is_compatible(&other.unit) {
            return Err("Incompatible units for subtraction");
        }

        let result_unit = if self.unit == Unit::None {
            other.unit.clone()
        } else {
            self.unit.clone()
        };

        Ok(Number::with_unit(
            self.value - other.value,
            result_unit,
            self.position.clone(),
        ))
    }

    /// Multiply two numbers
    pub fn multiply(&self, other: &Number) -> Number {
        let result_unit = if self.unit == Unit::None {
            other.unit.clone()
        } else if other.unit == Unit::None {
            self.unit.clone()
        } else {
            // For multiplication with both having units, keep the first unit
            self.unit.clone()
        };

        Number::with_unit(self.value * other.value, result_unit, self.position.clone())
    }

    /// Divide two numbers
    pub fn divide(&self, other: &Number) -> Result<Number, &'static str> {
        if other.value == 0.0 {
            return Err("Division by zero");
        }

        let result_unit = if other.unit == Unit::None {
            self.unit.clone()
        } else if self.unit == other.unit {
            Unit::None
        } else {
            self.unit.clone()
        };

        Ok(Number::with_unit(
            self.value / other.value,
            result_unit,
            self.position.clone(),
        ))
    }
}

impl NamedColor {
    /// Get the RGB values for a named color
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            NamedColor::Transparent => (0, 0, 0), // Special case, alpha = 0
            NamedColor::AliceBlue => (240, 248, 255),
            NamedColor::AntiqueWhite => (250, 235, 215),
            NamedColor::Aqua => (0, 255, 255),
            NamedColor::Aquamarine => (127, 255, 212),
            NamedColor::Azure => (240, 255, 255),
            NamedColor::Beige => (245, 245, 220),
            NamedColor::Bisque => (255, 228, 196),
            NamedColor::Black => (0, 0, 0),
            NamedColor::BlanchedAlmond => (255, 235, 205),
            NamedColor::Blue => (0, 0, 255),
            NamedColor::BlueViolet => (138, 43, 226),
            NamedColor::Brown => (165, 42, 42),
            NamedColor::BurlyWood => (222, 184, 135),
            NamedColor::CadetBlue => (95, 158, 160),
            NamedColor::Chartreuse => (127, 255, 0),
            NamedColor::Chocolate => (210, 105, 30),
            NamedColor::Coral => (255, 127, 80),
            NamedColor::CornflowerBlue => (100, 149, 237),
            NamedColor::Cornsilk => (255, 248, 220),
            NamedColor::Crimson => (220, 20, 60),
            NamedColor::Cyan => (0, 255, 255),
            NamedColor::DarkBlue => (0, 0, 139),
            NamedColor::DarkCyan => (0, 139, 139),
            NamedColor::DarkGoldenRod => (184, 134, 11),
            NamedColor::DarkGray => (169, 169, 169),
            NamedColor::DarkGreen => (0, 100, 0),
            NamedColor::DarkKhaki => (189, 183, 107),
            NamedColor::DarkMagenta => (139, 0, 139),
            NamedColor::DarkOliveGreen => (85, 107, 47),
            NamedColor::DarkOrange => (255, 140, 0),
            NamedColor::DarkOrchid => (153, 50, 204),
            NamedColor::DarkRed => (139, 0, 0),
            NamedColor::DarkSalmon => (233, 150, 122),
            NamedColor::DarkSeaGreen => (143, 188, 143),
            NamedColor::DarkSlateBlue => (72, 61, 139),
            NamedColor::DarkSlateGray => (47, 79, 79),
            NamedColor::DarkTurquoise => (0, 206, 209),
            NamedColor::DarkViolet => (148, 0, 211),
            NamedColor::DeepPink => (255, 20, 147),
            NamedColor::DeepSkyBlue => (0, 191, 255),
            NamedColor::DimGray => (105, 105, 105),
            NamedColor::DodgerBlue => (30, 144, 255),
            NamedColor::FireBrick => (178, 34, 34),
            NamedColor::FloralWhite => (255, 250, 240),
            NamedColor::ForestGreen => (34, 139, 34),
            NamedColor::Fuchsia => (255, 0, 255),
            NamedColor::Gainsboro => (220, 220, 220),
            NamedColor::GhostWhite => (248, 248, 255),
            NamedColor::Gold => (255, 215, 0),
            NamedColor::GoldenRod => (218, 165, 32),
            NamedColor::Gray => (128, 128, 128),
            NamedColor::Green => (0, 128, 0),
            NamedColor::GreenYellow => (173, 255, 47),
            NamedColor::HoneyDew => (240, 255, 240),
            NamedColor::HotPink => (255, 105, 180),
            NamedColor::IndianRed => (205, 92, 92),
            NamedColor::Indigo => (75, 0, 130),
            NamedColor::Ivory => (255, 255, 240),
            NamedColor::Khaki => (240, 230, 140),
            NamedColor::Lavender => (230, 230, 250),
            NamedColor::LavenderBlush => (255, 240, 245),
            NamedColor::LawnGreen => (124, 252, 0),
            NamedColor::LemonChiffon => (255, 250, 205),
            NamedColor::LightBlue => (173, 216, 230),
            NamedColor::LightCoral => (240, 128, 128),
            NamedColor::LightCyan => (224, 255, 255),
            NamedColor::LightGoldenRodYellow => (250, 250, 210),
            NamedColor::LightGray => (211, 211, 211),
            NamedColor::LightGreen => (144, 238, 144),
            NamedColor::LightPink => (255, 182, 193),
            NamedColor::LightSalmon => (255, 160, 122),
            NamedColor::LightSeaGreen => (32, 178, 170),
            NamedColor::LightSkyBlue => (135, 206, 250),
            NamedColor::LightSlateGray => (119, 136, 153),
            NamedColor::LightSteelBlue => (176, 196, 222),
            NamedColor::LightYellow => (255, 255, 224),
            NamedColor::Lime => (0, 255, 0),
            NamedColor::LimeGreen => (50, 205, 50),
            NamedColor::Linen => (250, 240, 230),
            NamedColor::Magenta => (255, 0, 255),
            NamedColor::Maroon => (128, 0, 0),
            NamedColor::MediumAquaMarine => (102, 205, 170),
            NamedColor::MediumBlue => (0, 0, 205),
            NamedColor::MediumOrchid => (186, 85, 211),
            NamedColor::MediumPurple => (147, 112, 219),
            NamedColor::MediumSeaGreen => (60, 179, 113),
            NamedColor::MediumSlateBlue => (123, 104, 238),
            NamedColor::MediumSpringGreen => (0, 250, 154),
            NamedColor::MediumTurquoise => (72, 209, 204),
            NamedColor::MediumVioletRed => (199, 21, 133),
            NamedColor::MidnightBlue => (25, 25, 112),
            NamedColor::MintCream => (245, 255, 250),
            NamedColor::MistyRose => (255, 228, 225),
            NamedColor::Moccasin => (255, 228, 181),
            NamedColor::NavajoWhite => (255, 222, 173),
            NamedColor::Navy => (0, 0, 128),
            NamedColor::OldLace => (253, 245, 230),
            NamedColor::Olive => (128, 128, 0),
            NamedColor::OliveDrab => (107, 142, 35),
            NamedColor::Orange => (255, 165, 0),
            NamedColor::OrangeRed => (255, 69, 0),
            NamedColor::Orchid => (218, 112, 214),
            NamedColor::PaleGoldenRod => (238, 232, 170),
            NamedColor::PaleGreen => (152, 251, 152),
            NamedColor::PaleTurquoise => (175, 238, 238),
            NamedColor::PaleVioletRed => (219, 112, 147),
            NamedColor::PapayaWhip => (255, 239, 213),
            NamedColor::PeachPuff => (255, 218, 185),
            NamedColor::Peru => (205, 133, 63),
            NamedColor::Pink => (255, 192, 203),
            NamedColor::Plum => (221, 160, 221),
            NamedColor::PowderBlue => (176, 224, 230),
            NamedColor::Purple => (128, 0, 128),
            NamedColor::RebeccaPurple => (102, 51, 153),
            NamedColor::Red => (255, 0, 0),
            NamedColor::RosyBrown => (188, 143, 143),
            NamedColor::RoyalBlue => (65, 105, 225),
            NamedColor::SaddleBrown => (139, 69, 19),
            NamedColor::Salmon => (250, 128, 114),
            NamedColor::SandyBrown => (244, 164, 96),
            NamedColor::SeaGreen => (46, 139, 87),
            NamedColor::SeaShell => (255, 245, 238),
            NamedColor::Sienna => (160, 82, 45),
            NamedColor::Silver => (192, 192, 192),
            NamedColor::SkyBlue => (135, 206, 235),
            NamedColor::SlateBlue => (106, 90, 205),
            NamedColor::SlateGray => (112, 128, 144),
            NamedColor::Snow => (255, 250, 250),
            NamedColor::SpringGreen => (0, 255, 127),
            NamedColor::SteelBlue => (70, 130, 180),
            NamedColor::Tan => (210, 180, 140),
            NamedColor::Teal => (0, 128, 128),
            NamedColor::Thistle => (216, 191, 216),
            NamedColor::Tomato => (255, 99, 71),
            NamedColor::Turquoise => (64, 224, 208),
            NamedColor::Violet => (238, 130, 238),
            NamedColor::Wheat => (245, 222, 179),
            NamedColor::White => (255, 255, 255),
            NamedColor::WhiteSmoke => (245, 245, 245),
            NamedColor::Yellow => (255, 255, 0),
            NamedColor::YellowGreen => (154, 205, 50),
        }
    }

    /// Get the alpha value for a named color
    pub fn alpha(&self) -> f64 {
        match self {
            NamedColor::Transparent => 0.0,
            _ => 1.0,
        }
    }

    /// Convert to Color struct
    pub fn to_color(&self, position: Position) -> Color {
        let (r, g, b) = self.to_rgb();
        Color::rgba(r, g, b, self.alpha(), position)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css())
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css())
    }
}

impl fmt::Display for StringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringType::Quoted(s) => write!(f, "\"{}\"", s),
            StringType::Unquoted(s) => write!(f, "{}", s),
            StringType::Url(s) => write!(f, "url({})", s),
        }
    }
}

/// Convert HSL to RGB
fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> (u8, u8, u8) {
    let h = hue % 360.0;
    let s = saturation.clamp(0.0, 1.0);
    let l = lightness.clamp(0.0, 1.0);

    if s == 0.0 {
        let gray = (l * 255.0) as u8;
        return (gray, gray, gray);
    }

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Convert RGB to HSL
fn rgb_to_hsl(red: u8, green: u8, blue: u8) -> (f64, f64, f64) {
    let r = red as f64 / 255.0;
    let g = green as f64 / 255.0;
    let b = blue as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }

    let saturation = if lightness < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let hue = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    (hue, saturation, lightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_parsing() {
        assert_eq!(Unit::from_str("px"), Some(Unit::Px));
        assert_eq!(Unit::from_str("em"), Some(Unit::Em));
        assert_eq!(Unit::from_str("%"), Some(Unit::Percent));
        assert_eq!(Unit::from_str("invalid"), None);
    }

    #[test]
    fn test_unit_compatibility() {
        assert!(Unit::Px.is_compatible(&Unit::Em));
        assert!(Unit::Deg.is_compatible(&Unit::Rad));
        assert!(!Unit::Px.is_compatible(&Unit::Deg));
        assert!(Unit::None.is_compatible(&Unit::Px));
    }

    #[test]
    fn test_color_creation() {
        let pos = Position::new(1, 1);
        let color = Color::rgb(255, 0, 0, pos);

        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
        assert_eq!(color.alpha, 1.0);
    }

    #[test]
    fn test_color_hex_parsing() {
        let pos = Position::new(1, 1);
        let color = Color::from_hex("#ff0000", pos).unwrap();

        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
    }

    #[test]
    fn test_color_operations() {
        let pos = Position::new(1, 1);
        let color = Color::rgb(100, 100, 100, pos);

        let lightened = color.lighten(20.0);
        let darkened = color.darken(20.0);

        // Basic sanity checks
        assert!(lightened.to_hsl().2 > color.to_hsl().2);
        assert!(darkened.to_hsl().2 < color.to_hsl().2);
    }

    #[test]
    fn test_number_operations() {
        let pos = Position::new(1, 1);
        let a = Number::with_unit(10.0, Unit::Px, pos.clone());
        let b = Number::with_unit(5.0, Unit::Px, pos);

        let sum = a.add(&b).unwrap();
        assert_eq!(sum.value, 15.0);
        assert_eq!(sum.unit, Unit::Px);

        let product = a.multiply(&b);
        assert_eq!(product.value, 50.0);
    }

    #[test]
    fn test_hsl_rgb_conversion() {
        // Test red
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert_eq!((r, g, b), (255, 0, 0));

        // Test green
        let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
        assert_eq!((r, g, b), (0, 255, 0));

        // Test blue
        let (r, g, b) = hsl_to_rgb(240.0, 1.0, 0.5);
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_named_color() {
        let pos = Position::new(1, 1);
        let red = NamedColor::Red.to_color(pos);

        assert_eq!(red.red, 255);
        assert_eq!(red.green, 0);
        assert_eq!(red.blue, 0);
        assert_eq!(red.alpha, 1.0);
    }

    #[test]
    fn test_transparent_color() {
        let alpha = NamedColor::Transparent.alpha();
        assert_eq!(alpha, 0.0);
    }
}
