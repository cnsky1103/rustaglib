mod utils;
mod audio_properties;
mod tag;
mod mpeg;
mod id3v1;
use std::path::Path;

use crate::{audio_properties::AudioProperties, tag::Tag};

pub trait AudioFile {
    fn new();

    fn tag(&self) -> Box<dyn Tag>;

    fn audio_properties(&self) -> Box<dyn AudioProperties>;
}
