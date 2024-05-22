use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use crate::bitmap::Bitmap;
use crate::IntSize;

#[repr(C)]
pub struct ImageFrameDescriptor {
    pub image: Rc<RefCell<Bitmap>>,
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
    decoder.size()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_is_animated(opaque_decoder: *mut c_void) -> bool {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.is_animated()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_loop_count(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.loop_count()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame_count(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.frame_count()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_first_animated_frame_index(opaque_decoder: *mut c_void) -> usize {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.first_animated_frame_index()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame_with_ideal_size(opaque_decoder: *mut c_void, frame_index: usize, ideal_size: Option<IntSize>) -> ImageFrameDescriptor {
    let mut decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.frame_with_ideal_size(frame_index, ideal_size).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_frame(opaque_decoder: *mut c_void, frame_index: usize) -> ImageFrameDescriptor {
    let mut decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.frame(frame_index).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn image_decoder_plugin_natural_frame_format(opaque_decoder: *mut c_void) -> NaturalFrameFormat {
    let decoder: Box<Box<dyn ImageDecoderPlugin>> = unsafe { Box::from_raw(opaque_decoder as *mut _) };
    decoder.natural_frame_format()
}
