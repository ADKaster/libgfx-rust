#![allow(dead_code)]

use std::ffi::c_void;
use std::io::Read;
use bytes::buf::Buf;
use static_assertions::const_assert;
use crate::imagedecoderplugin::{ImageDecoderPlugin, ImageFrameDescriptor};
use crate::{Color, IntSize};
use crate::bitmap::{Bitmap, BitmapFormat};

#[derive (Debug, PartialEq, Copy, Clone)]
enum TGADataType {
    None = 0,
    UncompressedColorMapped = 1,
    UncompressedRGB = 2,
    UncompressedBlackAndWhite = 3,
    RunLengthEncodedColorMapped = 9,
    RunLengthEncodedRGB = 10,
    CompressedBlackAndWhite = 11,
    CompressedColorMapped = 32,
    CompressedColorMappedFourPass = 33
}

#[repr(C, packed)]
pub struct TGAHeader {
    id_length: u8,
    color_map_type: u8,
    data_type_code: TGADataType,
    color_map_origin: i16,
    color_map_length: i16,
    color_map_depth: u8,
    x_origin: i16,
    y_origin: i16,
    width: u16,
    height: u16,
    bits_per_pixel: u8,
    image_descriptor: u8
}
const_assert!(std::mem::size_of::<TGAHeader>() == 18);

struct TGAPixelPacketHeader {
    raw: bool,
    pixels_count: u8,
}

pub struct TGAImageDecoderPlugin<'a> {
    context: TGALoadingContext<'a>
}

struct TGALoadingContext<'a> {
    header: TGAHeader,
    reader: bytes::buf::Reader<&'a [u8]>,
    bytes: &'a[u8],
    bitmap: Option<Bitmap>
}

impl<'a> TGAImageDecoderPlugin<'a> {
    pub fn create(bytes: &'a[u8]) -> Result<Self, String> {
        if !Self::validate_before_create(bytes) {
            return Err("Invalid TGA file".to_string());
        }
        let mut decoder = Self::new(bytes);
        decoder.decode_tga_header()?;
        Ok(decoder)
    }

    fn drop(&mut self) {
        panic!("TGAImageDecoderPlugin: drop");
    }

    fn new(bytes: &'a[u8]) -> Self {
        Self {
            context: TGALoadingContext {
                header: TGAHeader {
                    id_length: 0,
                    color_map_type: 0,
                    data_type_code: TGADataType::None,
                    color_map_origin: 0,
                    color_map_length: 0,
                    color_map_depth: 0,
                    x_origin: 0,
                    y_origin: 0,
                    width: 0,
                    height: 0,
                    bits_per_pixel: 0,
                    image_descriptor: 0
                },
                reader: Buf::reader(bytes),
                bytes,
                bitmap: None
            }
        }
    }

    fn validate_before_create(bytes: &[u8]) -> bool {
        let mut header_data = [0u8; std::mem::size_of::<TGAHeader>()];
        let mut reader = Buf::reader(bytes);
        if reader.read_exact(&mut header_data).is_err() {
            return false;
        }
        let header: TGAHeader = unsafe { std::mem::transmute(header_data) };
        if Self::ensure_header_validity(&header, bytes.len()).is_err() {
            return false;
        }
        true
    }

    fn ensure_header_validity(header: &TGAHeader, whole_image_size: usize) -> Result<(), String> {
        if (header.bits_per_pixel % 8) != 0 || header.bits_per_pixel < 8 || header.bits_per_pixel > 32 {
            return Err("Invalid bits per pixel".to_string());
        }
        let bytes_remaining = whole_image_size - std::mem::size_of::<TGAHeader>();
        if header.data_type_code == TGADataType::UncompressedRGB && (bytes_remaining < header.width as usize *  header.height as usize * header.bits_per_pixel as usize / 8) {
            return Err("Invalid image size".to_string());
        }
        Ok(())
    }

    fn decode_tga_header(&mut self) -> Result<(), String> {
        let mut header_data = [0u8; std::mem::size_of::<TGAHeader>()];
        if let Err(e) = self.context.reader.read_exact(&mut header_data)  {
            return Err(e.to_string());
        }
        self.context.header = unsafe { std::mem::transmute(header_data) };
        Self::ensure_header_validity(&self.context.header, self.context.bytes.len())?;
        Ok(())
    }
}

fn read_pixel_from_reader(reader: &mut bytes::buf::Reader<&[u8]>, bytes_size: usize) -> Result<Color, String> {
    // NOTE: We support 24-bit color pixels and 32-bit color pixels
    match bytes_size {
        3 => {
            let mut color_data: [u8; 3] = [0u8; 3];
            if let Err(e) = reader.read_exact(&mut color_data) {
                return Err(e.to_string());
            }
            Ok(Color::from_rgb(color_data[2], color_data[1], color_data[0]))
        }
        4 => {
            let mut color_data: [u8; 4] = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut color_data) {
                return Err(e.to_string());
            }
            Ok(Color::from_rgba(color_data[3], color_data[2], color_data[1], color_data[0]))
        }
        _ => {
            unreachable!("Invalid bytes size");
        }
    }
}

fn read_pixel_packet_header(reader: &mut bytes::buf::Reader<&[u8]>) -> Result<TGAPixelPacketHeader, String> {
    let mut header_data: [u8; 1] = [0u8; 1];
    if let Err(e) = reader.read_exact(&mut header_data) {
        return Err(e.to_string());
    }
    let raw = (header_data[0] & 0x80) == 0;
    let mut pixels_count = header_data[0] & 0x7F;
    // NOTE: Run-length-encoded/Raw pixel packets cannot encode zero pixels,
    // so value 0 stands for 1 pixel, 1 stands for 2, etc...
    pixels_count += 1;
    assert!(pixels_count > 0);
    Ok(TGAPixelPacketHeader {
        raw,
        pixels_count
    })
}

impl<'a> ImageDecoderPlugin for TGAImageDecoderPlugin<'a> {
    fn size(&self) -> IntSize {
        IntSize {
            width: self.context.header.width as i32,
            height: self.context.header.height as i32
        }
    }

    fn frame_with_ideal_size(&mut self, index: usize, _: Option<IntSize>) -> Result<ImageFrameDescriptor, String> {

        let bits_per_pixel = self.context.header.bits_per_pixel;
        let color_map = self.context.header.color_map_type;
        let data_type =  self.context.header.data_type_code;
        let width = self.context.header.width;
        let height =  self.context.header.height;
        let x_origin =  self.context.header.x_origin;
        let mut y_origin =  self.context.header.y_origin;

        if index != 0 {
            return Err("TAImageDecoderPlugin: frame index must be 0".to_string());
        }

        if color_map > 1 {
            return Err("TAImageDecoderPlugin: Invalid color map type".to_string());
        }

        if self.context.bitmap.is_some() {
            return Ok(ImageFrameDescriptor {
                image: self.context.bitmap.as_mut().unwrap().clone(),
                duration: 0
            });
        }

        let mut bitmap = match bits_per_pixel {
            24 => Bitmap::new(BitmapFormat::BGRx8888, IntSize { width: width as i32, height: height as i32 }, 1)?,
            32 => Bitmap::new(BitmapFormat::BGRA8888, IntSize { width: width as i32, height: height as i32 }, 1)?,
            _ => {
                // FIXME: Implement other TGA bit depths
                return Err("TGAImageDecoderPlugin: Can only handle 24 and 32 bits per pixel".to_string())
            }
        };

        // FIXME: Try to understand the Image origin (instead of X and Y origin coordinates)
        // based on the Image descriptor, Field 5.6, bits 4 and 5.

        // NOTE: If Y origin is set to a negative number, just assume the generating software
        // meant that we start with Y origin at the top height of the picture.
        // At least this is the observed behavior when generating some pictures in GIMP.
        if y_origin < 0 {
            y_origin = height as i16;
        }
        if y_origin != 0 && y_origin != height as i16 {
            return Err("TGAImageDecoderPlugin: Can only handle Y origin which is 0 or the entire height".to_string());
        }
        if x_origin != 0 && x_origin != width as i16 {
            return Err("TGAImageDecoderPlugin: Can only handle X origin which is 0 or the entire width".to_string());
        }

        assert_eq!(bits_per_pixel % 8, 0);
        let bytes_per_pixel = bits_per_pixel / 8;

        match data_type {
            TGADataType::UncompressedRGB => {
                for row in 0..height as i32 {
                    for col in 0..width as i32 {
                        let actual_row = if y_origin >= height as i16 { row } else { height as i32 - 1 - row };
                        let actual_col = if x_origin <= width as i16 { col } else { width as i32 - 1 - col };
                        let pixel = read_pixel_from_reader(&mut self.context.reader, bytes_per_pixel as usize)?;
                        bitmap.set_pixel(actual_col, actual_row, pixel.color);
                    }
                }
            },
            TGADataType::RunLengthEncodedRGB => {
                let mut pixel_index = 0usize;
                let pixel_count = height as usize * width as usize;
                while pixel_index < pixel_count {
                    let pixel_packet_header = read_pixel_packet_header(&mut self.context.reader)?;
                    assert!(pixel_packet_header.pixels_count > 0);

                    let mut pixel = read_pixel_from_reader(&mut self.context.reader, bytes_per_pixel as usize)?;
                    let max_pixel_index = std::cmp::min(pixel_index + pixel_packet_header.pixels_count as usize, pixel_count);
                    for current_pixel_index in pixel_index..max_pixel_index {
                        let row = current_pixel_index as i32 / width as i32;
                        let col = current_pixel_index as i32 % width as i32;
                        let actual_row = if y_origin >= height as i16 { row } else { height as i32 - 1 - row };
                        let actual_col = if x_origin <= width as i16 { col } else { width as i32 - 1 - col };
                        bitmap.set_pixel(actual_col, actual_row, pixel.color);
                        if pixel_packet_header.raw && current_pixel_index + 1 < max_pixel_index {
                            let next_pixel = read_pixel_from_reader(&mut self.context.reader, bytes_per_pixel as usize)?;
                            pixel = next_pixel;
                        }
                    }
                    pixel_index += pixel_packet_header.pixels_count as usize;
                }
            }
            _ => { return Err("TGAImageDecoderPlugin: Unsupported TGA data type".to_string()); }
        }

        self.context.bitmap = Some(bitmap.clone());

        Ok(ImageFrameDescriptor {
            image: bitmap,
            duration: 0
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn tga_image_decoder_plugin_new<'a>(bytes: *const u8, size: usize) -> *mut c_void {
    let bytes = unsafe {
        assert!(!bytes.is_null());
        std::slice::from_raw_parts(bytes, size)
    };
    match TGAImageDecoderPlugin::create(bytes) {
        Ok(decoder) => {
            let interface: Box<dyn ImageDecoderPlugin> = Box::new(decoder);
            let boxed_interface = Box::new(interface);
            Box::into_raw(boxed_interface) as *mut c_void
        },
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn tga_image_decoder_plugin_free(opaque_decoder: *mut c_void) {
    if !opaque_decoder.is_null() {
        let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
        drop(decoder);
    }
}
