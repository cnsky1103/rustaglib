use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom},
};

use crate::{utils::byte_vec_find, AudioFile};

use super::{
    audio_properties::{AudioProperties, ReadStyle},
    tag::{PropertyMap, Tag},
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub struct MpegFile {
    pub tag: MpegTag,
    pub audio_properties: MpegProperties,
}

impl AudioFile for MpegFile {
    fn new() {}

    fn tag(&self) -> Box<dyn Tag> {
        Box::from(self.tag.clone())
    }

    fn audio_properties(&self) -> Box<dyn AudioProperties> {
        Box::from(self.audio_properties.clone())
    }
}

#[derive(Clone)]
pub struct MpegTag {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    comment: Option<String>,
    genre: Option<String>,
    year: Option<u32>,
    track: Option<u32>,
    property_map: PropertyMap,
}

impl Tag for MpegTag {
    fn properties(&self) -> &PropertyMap {
        &self.property_map
    }

    fn remove_unsupported_properties(&mut self, properties: Vec<String>) {
        for s in properties {
            self.property_map.remove(&s);
        }
    }

    fn set_properties(&mut self, properties: PropertyMap) {
        self.property_map = properties;
    }

    fn title(&self) -> &Option<String> {
        &self.title
    }

    fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    fn artist(&self) -> &Option<String> {
        &self.artist
    }

    fn set_artist(&mut self, artist: Option<String>) {
        self.artist = artist;
    }

    fn album(&self) -> &Option<String> {
        &self.album
    }

    fn set_album(&mut self, album: Option<String>) {
        self.album = album;
    }

    fn comment(&self) -> &Option<String> {
        &self.comment
    }

    fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }

    fn genre(&self) -> &Option<String> {
        &self.genre
    }

    fn set_genre(&mut self, genre: Option<String>) {
        self.genre = genre;
    }

    fn year(&self) -> &Option<u32> {
        &self.year
    }

    fn set_year(&mut self, year: Option<u32>) {
        self.year = year
    }

    fn track(&self) -> &Option<u32> {
        &self.track
    }

    fn set_track(&mut self, track: Option<u32>) {
        self.track = track;
    }

    fn is_empty(&self) -> bool {
        return self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.comment.is_none()
            && self.genre.is_none()
            && self.year.is_none()
            && self.track.is_none();
    }
}

#[derive(Clone)]
pub struct MpegProperties {
    d: MpegPropertiesPrivate,
}

#[derive(Clone)]
pub(crate) struct MpegPropertiesPrivate {
    xing_header: XingHeader,
    length: u32,
    bitrate: u32,
    sample_rate: u32,
    channels: u32,
    layer: u32,
    version: Version,
    channel_mode: ChannelMode,
    protection_enabled: bool,
    is_copyrighted: bool,
    is_original: bool,
    read_style: ReadStyle,
}

impl AudioProperties for MpegProperties {
    fn length(&self) -> u32 {
        self.d.length
    }

    fn length_in_seconds(&self) -> u32 {
        self.d.length / 1000
    }

    fn length_in_milliseconds(&self) -> u32 {
        self.d.length
    }

    fn bitrate(&self) -> u32 {
        self.d.bitrate
    }

    fn sample_rate(&self) -> u32 {
        self.d.sample_rate
    }

    fn channels(&self) -> u32 {
        self.d.channels
    }
}

/* impl MpegProperties {
    pub(crate) fn new(file: File, style: ReadStyle) -> Result<Self> {
        // Only the first valid frame is required if we have a VBR header.
    }
} */

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub(crate) enum Version {
    // MPEG Version 1
    Version1 = 0,
    // MPEG Version 2
    Version2 = 1,
    // MPEG Version 2.5
    Version2_5 = 2,
}

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub(crate) enum ChannelMode {
    // Stereo
    Stereo = 0,
    // Stereo
    JointStereo = 1,
    // Dual Mono
    DualChannel = 2,
    // Mono
    SingleChannel = 3,
}
struct MpegHeaderPrivate {
    is_valid: bool,
    version: Version,
    layer: u32,
    protection_enabled: bool,
    bitrate: u32,
    sample_rate: u32,
    is_padded: bool,
    channel_mode: ChannelMode,
    is_copyrighted: bool,
    is_original: bool,
    frame_length: u32,
    samples_per_frame: u32,
}

impl Default for MpegHeaderPrivate {
    fn default() -> Self {
        Self {
            is_valid: false,
            version: Version::Version1,
            layer: 0,
            protection_enabled: false,
            bitrate: 0,
            sample_rate: 0,
            is_padded: false,
            channel_mode: ChannelMode::Stereo,
            is_copyrighted: false,
            is_original: false,
            frame_length: 0,
            samples_per_frame: 0,
        }
    }
}

pub struct MpegHeader {
    d: MpegHeaderPrivate,
}

impl MpegHeader {
    pub(crate) fn new(mut file: File, offset: u64, check_length: bool) -> Result<Self> {
        let mut header = MpegHeader {
            d: MpegHeaderPrivate::default(),
        };

        file.seek(SeekFrom::Start(offset))?;
        let mut data = [0 as u8; 4];
        file.read_exact(&mut data)?;

        // Check for the MPEG synch bytes.
        if !is_frame_sync(data.to_vec(), 0) {
            return Err(Error::new(
                ErrorKind::Other,
                "MPEG::Header::parse() -- MPEG header did not match MPEG synch.",
            ));
        }

        // set the MPEG version
        let version_bits = (data[1] >> 3) & 0x03;
        if version_bits == 0 {
            header.d.version = Version::Version2_5;
        } else if version_bits == 2 {
            header.d.version = Version::Version2;
        } else if version_bits == 3 {
            header.d.version = Version::Version1;
        } else {
            return Ok(header);
        }

        // set the MPEG layer

        let layer_bits = (data[1] >> 1) & 0x03;

        header.d.layer = match layer_bits {
            1 => 3,
            2 => 2,
            3 => 1,
            _ => return Ok(header), // TODO: check if this style is ok
        };

        header.d.protection_enabled = (data[1] & 0x01) == 0;

        // Set the bitrate
        let bitrates: [[[u32; 16]; 3]; 2] = [
            [
                // Version 1
                [
                    0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0,
                ], // layer 1
                [
                    0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0,
                ], // layer 2
                [
                    0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
                ], // layer 3
            ],
            [
                // Version 2 or 2.5
                [
                    0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0,
                ], // layer 1
                [
                    0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
                ], // layer 2
                [
                    0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
                ], // layer 3
            ],
        ];

        let version_index = match header.d.version {
            Version::Version1 => 0,
            _ => 1,
        };

        let layer_index = if header.d.layer > 0 {
            header.d.layer - 1
        } else {
            0
        };

        // The bitrate index is encoded as the first 4 bits of the 3rd byte,
        // i.e. 1111xxxx
        let bitrate_index = (data[2] >> 4) & 0x0f;

        header.d.bitrate = bitrates[version_index][layer_index as usize][bitrate_index as usize];

        if header.d.bitrate == 0 {
            return Ok(header);
        }

        // set the sample rate

        let sample_rates: [[u32; 4]; 3] = [
            [44100, 48000, 32000, 0], // Version 1
            [22050, 24000, 16000, 0], // Version 2
            [11025, 12000, 8000, 0],  // Version 2.5
        ];

        // The sample rate index is encoded as two bits in the 3nd byte, i.e. xxxx11xx
        let samplerate_index = (data[2] >> 2) & 0x03;

        let v: u8 = header.d.version.into();
        header.d.sample_rate = sample_rates[v as usize][samplerate_index as usize];

        if header.d.sample_rate == 0 {
            return Ok(header);
        }

        // The channel mode is encoded as a 2 bit value at the end of the 3nd byte,
        // i.e. xxxxxx11

        header.d.channel_mode = ChannelMode::try_from((data[3] >> 6) & 0x03).unwrap();

        header.d.is_original = (data[3] & 0x04) != 0;
        header.d.is_copyrighted = (data[3] & 0x08) != 0;
        header.d.is_padded = (data[2] & 0x02) != 0;

        // samples per frame
        let samples_per_frame: [[u32; 2]; 3] = [
            // MPEG1, 2/2.5
            [384, 384],   // Layer I
            [1152, 1152], // Layer II
            [1152, 576],  // Layer III
        ];

        header.d.samples_per_frame = samples_per_frame[layer_index as usize][version_index];

        // calculate the frame length

        let padding_size: [u32; 3] = [4, 1, 1];
        header.d.frame_length =
            header.d.samples_per_frame * header.d.bitrate * 125 / header.d.sample_rate;

        if header.d.is_padded {
            header.d.frame_length += padding_size[layer_index as usize];
        }

        if check_length {
            // Check if the frame length has been calculated correctly, or the next frame
            // header is right next to the end of this frame.

            // The MPEG versions, layers and sample rates of the two frames should be
            // consistent. Otherwise, we assume that either or both of the frames are
            // broken.

            file.seek(SeekFrom::Start(offset + header.d.frame_length as u64))?;

            let mut next_data = [0u8; 4];
            file.read_exact(&mut next_data)?;

            let header_mask: u32 = 0xfffe0c00;
            let header_n: u32 = u32::from_be_bytes(data) & header_mask;
            let next_header_n: u32 = u32::from_be_bytes(next_data) & header_mask;

            if header_n != next_header_n {
                return Ok(header);
            }
        }

        // Now that we're done parsing, set this to be a valid frame.
        header.d.is_valid = true;

        Ok(header)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.d.is_valid
    }

    pub(crate) fn version(&self) -> Version {
        self.d.version
    }

    pub(crate) fn layer(&self) -> u32 {
        self.d.layer
    }

    pub(crate) fn protection_enabled(&self) -> bool {
        self.d.protection_enabled
    }

    pub(crate) fn bitrate(&self) -> u32 {
        self.d.bitrate
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.d.sample_rate
    }

    pub(crate) fn is_padded(&self) -> bool {
        self.d.is_padded
    }

    pub(crate) fn channel_mode(&self) -> ChannelMode {
        self.d.channel_mode
    }

    pub(crate) fn is_copyrighted(&self) -> bool {
        self.d.is_copyrighted
    }

    pub(crate) fn is_original(&self) -> bool {
        self.d.is_original
    }

    pub(crate) fn frame_length(&self) -> u32 {
        self.d.frame_length
    }

    pub(crate) fn samples_per_frame(&self) -> u32 {
        self.d.samples_per_frame
    }
}

fn is_frame_sync(bytes: Vec<u8>, offset: usize) -> bool {
    let b1 = bytes[offset + 0];
    let b2 = bytes[offset + 1];

    b1 == 0xff && b2 != 0xff && (b2 & 0xe0) == 0xe0
}

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum XingHeaderType {
    // Invalid header or no VBR header found.
    Invalid = 0,
    // Xing header.
    Xing = 1,
    // VBRI header.
    VBRI = 2,
}

#[derive(Clone)]
pub(crate) struct XingHeader {
    d: XingHeaderPrivate,
}

#[derive(Clone)]
pub(crate) struct XingHeaderPrivate {
    frames: u32,
    size: u32,
    header_type: XingHeaderType,
}

impl Default for XingHeaderPrivate {
    fn default() -> Self {
        Self {
            frames: 0,
            size: 0,
            header_type: XingHeaderType::Invalid,
        }
    }
}

impl XingHeader {
    pub(crate) fn new(data: Vec<u8>) -> Result<Self> {
        // look for a Xing header
        let mut xing_header = Self {
            d: XingHeaderPrivate::default(),
        };

        let mut offset = byte_vec_find(&data, &vec![b'X', b'i', b'n', b'g'], 0, 1);

        if offset.is_none() {
            offset = byte_vec_find(&data, &vec![b'I', b'n', b'f', b'o'], 0, 1);
        }

        match offset {
            Some(offset) => {
                if data.len() < offset + 16 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "MPEG::XingHeader::parse() -- Xing header found but too short.",
                    ));
                }

                if (data[offset + 7] & 0x03) != 0x03 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "MPEG::XingHeader::parse() -- Xing header doesn't contain the required information.",
                    ));
                }

                xing_header.d.frames =
                    u32::from_be_bytes(data[offset + 8..offset + 12].try_into().unwrap());

                xing_header.d.size =
                    u32::from_be_bytes(data[offset + 12..offset + 16].try_into().unwrap());

                xing_header.d.header_type = XingHeaderType::Xing;
            }
            None => {
                let offset = byte_vec_find(&data, &vec![b'V', b'B', b'R', b'I'], 0, 1);

                if offset.is_some() {
                    let offset = offset.unwrap();

                    // VBRI header found

                    if data.len() < offset + 32 {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "MPEG::XingHeader::parse() -- VBRI header found but too short.",
                        ));
                    }

                    xing_header.d.frames =
                        u32::from_be_bytes(data[offset + 14..offset + 18].try_into().unwrap());
                    xing_header.d.size =
                        u32::from_be_bytes(data[offset + 10..offset + 14].try_into().unwrap());
                    xing_header.d.header_type = XingHeaderType::VBRI;
                }
            }
        }

        Ok(xing_header)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.d.header_type != XingHeaderType::Invalid && self.d.frames > 0 && self.d.size > 0
    }

    pub(crate) fn total_frames(&self) -> u32 {
        self.d.frames
    }

    pub(crate) fn total_size(&self) -> u32 {
        self.d.size
    }

    pub(crate) fn header_type(&self) -> XingHeaderType {
        self.d.header_type
    }
}
