#![allow(dead_code)]

pub mod tgaloader;
pub mod imagedecoderplugin;
pub mod bitmap;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IntSize {
    pub width: i32,
    pub height: i32
}

impl IntSize {
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

pub type ARGB = u32;

#[derive(Debug)]
pub struct Color {
    color: ARGB
}

impl Color {
    fn new() -> Self {
        Self { color: 0 }
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { color: 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }
    }

    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { color: ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }
    }
}

impl From<ARGB> for Color {
    fn from(color: ARGB) -> Self {
        Self { color }
    }
}
