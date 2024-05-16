use std::cell::RefCell;
use std::rc::Rc;
use crate::bitmap::Bitmap;
use crate::IntSize;

pub struct ImageFrameDescriptor {
    pub image: Rc<RefCell<Bitmap>>,
    pub duration: i32
}

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
