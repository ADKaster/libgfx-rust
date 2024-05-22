use std::ffi::c_void;
use std::mem::ManuallyDrop;
use crate::bitmap::{Bitmap, BitmapFormat};
use crate::IntSize;

pub struct ImageFrameDescriptor {
    pub image: Bitmap,
    pub duration: i32
}

#[repr(C)]
#[derive (Debug, PartialEq)]
pub enum NaturalFrameFormat {
    RGB,
    Grayscale,
    CMYK,
    Vector,
}

#[repr(C)]
pub struct FFIBuffer {
    pub data: *mut u8,
    pub size: usize,
    pub capacity: usize
}

impl From<Vec<u8>> for FFIBuffer {
    fn from(data: Vec<u8>) -> Self {
        let mut v = ManuallyDrop::new(data);

        let capacity = v.capacity();
        let size = v.len();
        let data = v.as_mut_ptr();
        FFIBuffer { data, size, capacity }
    }
}

#[repr(C)]
pub struct FFIBitmap {
    pub format: BitmapFormat,
    pub size: IntSize,
    pub scale: i32,
    pub pitch: u32,
    pub data: FFIBuffer
}

impl From<Bitmap> for FFIBitmap {
    fn from(bitmap: Bitmap) -> Self {
        FFIBitmap {
            format: bitmap.format,
            size: bitmap.size,
            scale: bitmap.scale,
            pitch: bitmap.pitch,
            data: bitmap.data.into(),
        }
    }
}

#[repr(C)]
pub struct FFIImageFrameDescriptor {
    pub image: FFIBitmap,
    pub duration: i32
}

impl From<ImageFrameDescriptor> for FFIImageFrameDescriptor {
    fn from(descriptor: ImageFrameDescriptor) -> Self {
        FFIImageFrameDescriptor {
            image: descriptor.image.into(),
            duration: descriptor.duration
        }
    }
}

pub trait ImageDecoderPlugin {
    fn size(&self) -> IntSize;
    fn is_animated(&self) -> bool { false }
    fn loop_count(&self) -> usize { 0 }
    fn frame_count(&self) -> usize { 1 }
    fn first_animated_frame_index(&self) -> usize { 0 }
    fn frame_with_ideal_size(&mut self, frame_index: usize, ideal_size: Option<IntSize>) -> Result<ImageFrameDescriptor, String>;
    fn frame(&mut self, frame_index: usize) -> Result<ImageFrameDescriptor, String> {
        self.frame_with_ideal_size(frame_index, None)
    }
    // FIXME: Metadata
    // FIXME: ICC data
    fn natural_frame_format(&self) -> NaturalFrameFormat { NaturalFrameFormat::RGB }
    // FIXME: CMYK Frame
    // FIXME: Vector Frame
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_size(opaque_decoder: *mut c_void) -> IntSize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let size = decoder.size();
    std::mem::forget(decoder);
    size
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_is_animated(opaque_decoder: *mut c_void) -> bool {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let is_animated = decoder.is_animated();
    std::mem::forget(decoder);
    is_animated
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_loop_count(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let loop_count  = decoder.loop_count();
    std::mem::forget(decoder);
    loop_count
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame_count(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let frame_count = decoder.frame_count();
    std::mem::forget(decoder);
    frame_count
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_first_animated_frame_index(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let index = decoder.first_animated_frame_index();
    std::mem::forget(decoder);
    index
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame_with_ideal_size(opaque_decoder: *mut c_void, frame_index: usize, ideal_size: *const IntSize) -> FFIImageFrameDescriptor {
    let mut decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let ideal_size = if ideal_size.is_null() {
        None
    } else {
        Some(unsafe { *ideal_size })
    };
    let frame = decoder.frame_with_ideal_size(frame_index, ideal_size).unwrap().into();
    std::mem::forget(decoder);
    frame
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_free_frame(ffiimage_frame_descriptor: FFIImageFrameDescriptor) {
    let ffi_buffer = ffiimage_frame_descriptor.image.data;
    let _ = Vec::from_raw_parts(ffi_buffer.data, ffi_buffer.size, ffi_buffer.capacity);
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame(opaque_decoder: *mut c_void, frame_index: usize) -> FFIImageFrameDescriptor {
    let mut decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let frame = decoder.frame(frame_index).unwrap();
    std::mem::forget(decoder);
    frame.into()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_natural_frame_format(opaque_decoder: *mut c_void) -> NaturalFrameFormat {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    let format = decoder.natural_frame_format();
    std::mem::forget(decoder);
    format
}
