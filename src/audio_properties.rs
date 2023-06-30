pub trait AudioProperties {
    // returns the length of the file in seconds
    fn length(&self) -> u32;

    // returns the length of the file in seconds
    fn length_in_seconds(&self) -> u32;

    // returns the length of the file in milliseconds
    fn length_in_milliseconds(&self) -> u32;

    // returns the most appropriate bit rate for the file in kb/s.  For constant
    // bitrate formats this is simply the bitrate of the file.  For variable
    // bitrate formats this is either the average or nominal bitrate.
    fn bitrate(&self) -> u32;

    // returns the sample rate in Hz
    fn sample_rate(&self) -> u32;

    // returns the number of audio channels
    fn channels(&self) -> u32;
}

#[derive(Clone, Copy)]
pub(crate) enum ReadStyle {
    // Read as little of the file as possible
    Fast,
    // Read more of the file and make better values guesses
    Average,
    // Read as much of the file as needed to report accurate values
    Accurate,
}