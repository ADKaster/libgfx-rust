use std::cell::RefCell;
use std::rc::Rc;
use crate::{ARGB, IntSize};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BitmapFormat {
    Invalid = 0,
    BGRx8888 = 1,
    BGRA8888 = 2,
    RGBA8888 = 3,
}

impl BitmapFormat {
    pub fn is_valid(format: u32) -> bool {
        matches!(format, 0..=3)
    }
}

#[repr(u8)]
pub enum StorageFormat {
    BGRx8888,
    BGRA8888,
    RGBA8888
}

impl StorageFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            StorageFormat::BGRx8888 => 4,
            StorageFormat::BGRA8888 => 4,
            StorageFormat::RGBA8888 => 4,
        }
    }
}

impl From<BitmapFormat> for StorageFormat {
    fn from(format: BitmapFormat) -> Self {
        match format {
            BitmapFormat::BGRx8888 => StorageFormat::BGRx8888,
            BitmapFormat::BGRA8888 => StorageFormat::BGRA8888,
            BitmapFormat::RGBA8888 => StorageFormat::RGBA8888,
            _ => panic!("Invalid bitmap format")
        }
    }
}

#[repr(C)]
pub enum RotationDirection {
    CounterClockwise,
    Flip,
    Clockwise
}

#[repr(C)]
pub struct Bitmap {
    format: BitmapFormat,
    size: IntSize,
    scale: i32,
    pitch: u32,
    data: Vec<u8>,
}

impl Bitmap {
    pub fn new(bitmap_format: BitmapFormat, size: IntSize, intrinsic_scale: i32) -> Result<Rc<RefCell<Self>>, String> {
        Ok(Rc::new(RefCell::new(Self {
            format: bitmap_format,
            size,
            scale: intrinsic_scale,
            pitch: 0,
            data: Self::allocate_backing_store(bitmap_format, size, intrinsic_scale)?
        })))
    }

    pub fn minimum_pitch(physical_width: usize, format: BitmapFormat) -> usize {
        let format = StorageFormat::from(format);
        physical_width * format.bytes_per_pixel() as usize
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: ARGB) {
        let offset = y as usize * self.pitch as usize + x as usize * std::mem::size_of::<ARGB>();
        let color = color.to_le_bytes();
        self.data[offset..offset + 4].copy_from_slice(&color);
    }

    fn size_would_overflow(format: BitmapFormat, size: IntSize, scale_factor: i32) -> bool {
        if size.is_empty() {
            return true;
        }
        // This check is a bit arbitrary, but should protect us from most shenanigans:
        if size.width > i16::MAX as i32 || size.height > i16::MAX as i32 || !(1..=4).contains(&scale_factor) {
            return true;
        }
        // In contrast, this check is absolutely necessary:
        let pitch = Self::minimum_pitch(size.width as usize * scale_factor as usize, format);
        let (_, overflows) = usize::overflowing_mul(pitch, size.height as usize * scale_factor as usize);
        overflows
    }

    fn allocate_backing_store(format: BitmapFormat, size: IntSize, scale: i32) -> Result<Vec<u8>, String> {
        if size.is_empty() {
            return Err("Bitmap::allocate_backing_store: size is empty".to_string());
        }
        if Self::size_would_overflow(format, size, scale) {
            return Err("Bitmap::allocate_backing_store: size would overflow".to_string());
        }

        let pitch = Self::minimum_pitch(size.width as usize * scale as usize, format);
        let data_size_in_bytes = pitch * size.height as usize * scale as usize;

        Ok(vec![0u8; data_size_in_bytes])
    }
}
