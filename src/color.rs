use std::str::FromStr;

use thiserror::Error;

#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn grey(v: f32) -> Self {
        Self {
            r: v,
            g: v,
            b: v,
            a: v,
        }
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<f32> = s
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<_, _>>()
            .map_err(|_e| ColorParseError::InvalidColor(s.into()))?;

        match numbers.len() {
            1 => Ok(Self::grey(numbers[0])),
            3 => Ok(Self::rgb(numbers[0], numbers[1], numbers[2])),
            4 => Ok(Self::rgba(numbers[0], numbers[1], numbers[2], numbers[3])),
            _ => Err(ColorParseError::InvalidColor(s.into())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ColorParseError {
    #[error("The color {0} could not be parsed")]
    InvalidColor(String),
}
