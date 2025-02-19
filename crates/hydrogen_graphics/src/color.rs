use std::ops::{Mul, MulAssign};

use derive_more::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    PartialEq,
)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for RGBA {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Mul for RGBA {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl MulAssign for RGBA {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<(f32, f32, f32)> for RGBA {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.1,
            a: 1.0,
        }
    }
}

impl From<RGBA> for [f32; 4] {
    fn from(value: RGBA) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

impl RGBA {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const DARK_BLUE: Self = Self::rgb(0.0, 0.0, 0.666);
    pub const DARK_GREEN: Self = Self::rgb(0.0, 0.666, 0.0);
    pub const DARK_AQUA: Self = Self::rgb(0.0, 0.666, 0.666);
    pub const DARK_RED: Self = Self::rgb(0.666, 0.0, 0.0);
    pub const DARK_PURPLE: Self = Self::rgb(0.666, 0.0, 0.666);
    pub const GOLD: Self = Self::rgb(1.0, 0.666, 0.0);
    pub const GRAY: Self = Self::rgb(0.666, 0.666, 0.666);
    pub const DARK_GRAY: Self = Self::rgb(0.333, 0.333, 0.333);
    pub const BLUE: Self = Self::rgb(0.333, 0.333, 1.0);
    pub const GREEN: Self = Self::rgb(0.333, 1.0, 0.333);
    pub const AQUA: Self = Self::rgb(0.333, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.333, 0.333);
    pub const LIGHT_PURPLE: Self = Self::rgb(1.0, 0.333, 1.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.333);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    pub const INVISIBLE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn with_red(mut self, r: f32) -> Self {
        self.r = r;
        self
    }

    pub const fn with_green(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    pub const fn with_blue(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    pub const fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    pub fn shadow(self) -> Self {
        self.mul_color(0.125)
    }

    pub fn is_visible(self) -> bool {
        self.a > (1.0 / 255.0) / 2.0
    }

    pub fn mul_color(self, scalar: f32) -> Self {
        Self {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            a: self.a,
        }
    }
}
