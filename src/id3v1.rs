use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek},
};

use crate::tag::Tag;

pub(crate) struct ID3v1TagPrivate {
    tag_offset: usize,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<u32>,
    comment: Option<String>,
    track: Option<u32>,
    genre: Option<String>,
}

pub(crate) struct ID3v1Tag {
    d: ID3v1TagPrivate,
}

impl Tag for ID3v1Tag {
    fn properties(&self) -> &crate::tag::PropertyMap {
        todo!()
    }

    fn remove_unsupported_properties(&mut self, properties: Vec<String>) {
        todo!()
    }

    fn set_properties(&mut self, properties: crate::tag::PropertyMap) {
        todo!()
    }

    fn title(&self) -> &Option<String> {
        &self.d.title
    }

    fn artist(&self) -> &Option<String> {
        &self.d.artist
    }

    fn album(&self) -> &Option<String> {
        &self.d.album
    }

    fn comment(&self) -> &Option<String> {
        &self.d.comment
    }

    fn genre(&self) -> &Option<String> {
        todo!()
    }

    fn year(&self) -> &Option<u32> {
        &self.d.year
    }

    fn track(&self) -> &Option<u32> {
        &self.d.track
    }

    fn set_title(&mut self, title: Option<String>) {
        self.d.title = title
    }

    fn set_artist(&mut self, artist: Option<String>) {
        self.d.artist = artist
    }

    fn set_album(&mut self, album: Option<String>) {
        self.d.album = album
    }

    fn set_comment(&mut self, comment: Option<String>) {
        self.d.comment = comment
    }

    fn set_genre(&mut self, genre: Option<String>) {
        todo!()
    }

    fn set_year(&mut self, year: Option<u32>) {
        self.d.year = year
    }

    fn set_track(&mut self, track: Option<u32>) {
        self.d.track = match track {
            Some(t) if t < 256 => track,
            _ => None,
        }
    }

    fn is_empty(&self) -> bool {
        todo!()
    }
}

impl ID3v1Tag {
    pub(crate) fn new(mut file: File, tag_offset: usize) -> std::io::Result<Self> {
        let mut d = ID3v1TagPrivate {
            tag_offset,
            title: None,
            artist: None,
            album: None,
            year: None,
            comment: None,
            track: Some(0),
            genre: Some(String::from("")),
        };

        file.seek(std::io::SeekFrom::Start(d.tag_offset.try_into().unwrap()))?;

        // read the tag, always 128 bytes
        let mut data = [0u8; 128];
        file.read_exact(&mut data)?;

        if data[0] != b'T' || data[1] != b'A' || data[2] != b'G' {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ID3v1 tag is not valid or could not be read at the specified offset.",
            ));
        }

        let mut offset = 3;
        d.title = Some(parse(
            String::from_utf8(data[offset..offset + 30].to_vec()).unwrap(),
        ));
        offset += 30;

        d.artist = Some(parse(
            String::from_utf8(data[offset..offset + 30].to_vec()).unwrap(),
        ));
        offset += 30;

        d.album = Some(parse(
            String::from_utf8(data[offset..offset + 30].to_vec()).unwrap(),
        ));
        offset += 30;

        d.year = Some(
            parse(String::from_utf8(data[offset..offset + 4].to_vec()).unwrap())
                .parse::<u32>()
                .unwrap(),
        );
        offset += 4;

        // Check for ID3v1.1 -- Note that ID3v1 *does not* support "track zero" -- this
        // is not a bug in TagLib.  Since a zeroed byte is what we would expect to
        // indicate the end of a C-String, specifically the comment string, a value of
        // zero must be assumed to be just that.

        if data[offset + 28] == 0 && data[offset + 29] != 0 {
            // ID3v1.1 detected
            d.comment = Some(parse(
                String::from_utf8(data[offset..offset + 28].to_vec()).unwrap(),
            ));
            d.track = Some(data[offset + 29] as u32);
        } else {
            d.comment = Some(parse(
                String::from_utf8(data[offset..offset + 30].to_vec()).unwrap(),
            ));
        }

        offset += 30;

        //d.genre = Some(data[offset] as u32);
        d.genre = Some(String::from(genre(data[offset] as usize)));

        Ok(Self { d })
    }

    pub(crate) fn set_genre_number(&mut self, i: Option<u32>) {
        let x = match i {
            Some(n) if n < 256 => n,
            _ => 255,
        };

        self.d.genre = Some(String::from(genre(x as usize)));
    }

    // probably useless
    /* pub(crate) fn get_genre_number(&self) -> usize {
        //genre_index(self.d.genre.unwrap().as_str())
        if self.d.genre.is_some() {
            let s = self.d.genre.clone();

            return genre_index(s.unwrap().as_str());
        }

        255
    } */

    pub(crate) fn render(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        data.append(&mut vec![b'T', b'A', b'G']);
        let s = self.d.title.clone();
        data.append(&mut resize(&s.unwrap().as_bytes().to_vec(), 30));

        let s = self.d.artist.clone();
        data.append(&mut resize(&s.unwrap().as_bytes().to_vec(), 30));

        let s = self.d.album.clone();
        data.append(&mut resize(&s.unwrap().as_bytes().to_vec(), 30));

        let s = self.d.year.clone();
        data.append(&mut resize(&s.unwrap().to_string().as_bytes().to_vec(), 4));

        let s = self.d.comment.clone();
        data.append(&mut resize(&s.unwrap().as_bytes().to_vec(), 28));

        data.push(0);
        data.push(self.d.track.unwrap() as u8);

        let s = self.d.genre.clone();
        data.push(genre_index(s.unwrap().as_str()) as u8);

        data
    }
}

const genres: &'static [&'static str] = &[
    "Blues",
    "Classic Rock",
    "Country",
    "Dance",
    "Disco",
    "Funk",
    "Grunge",
    "Hip-Hop",
    "Jazz",
    "Metal",
    "New Age",
    "Oldies",
    "Other",
    "Pop",
    "R&B",
    "Rap",
    "Reggae",
    "Rock",
    "Techno",
    "Industrial",
    "Alternative",
    "Ska",
    "Death Metal",
    "Pranks",
    "Soundtrack",
    "Euro-Techno",
    "Ambient",
    "Trip-Hop",
    "Vocal",
    "Jazz-Funk",
    "Fusion",
    "Trance",
    "Classical",
    "Instrumental",
    "Acid",
    "House",
    "Game",
    "Sound Clip",
    "Gospel",
    "Noise",
    "Alternative Rock",
    "Bass",
    "Soul",
    "Punk",
    "Space",
    "Meditative",
    "Instrumental Pop",
    "Instrumental Rock",
    "Ethnic",
    "Gothic",
    "Darkwave",
    "Techno-Industrial",
    "Electronic",
    "Pop-Folk",
    "Eurodance",
    "Dream",
    "Southern Rock",
    "Comedy",
    "Cult",
    "Gangsta",
    "Top 40",
    "Christian Rap",
    "Pop/Funk",
    "Jungle",
    "Native American",
    "Cabaret",
    "New Wave",
    "Psychedelic",
    "Rave",
    "Showtunes",
    "Trailer",
    "Lo-Fi",
    "Tribal",
    "Acid Punk",
    "Acid Jazz",
    "Polka",
    "Retro",
    "Musical",
    "Rock & Roll",
    "Hard Rock",
    "Folk",
    "Folk Rock",
    "National Folk",
    "Swing",
    "Fast Fusion",
    "Bebop",
    "Latin",
    "Revival",
    "Celtic",
    "Bluegrass",
    "Avant-garde",
    "Gothic Rock",
    "Progressive Rock",
    "Psychedelic Rock",
    "Symphonic Rock",
    "Slow Rock",
    "Big Band",
    "Chorus",
    "Easy Listening",
    "Acoustic",
    "Humour",
    "Speech",
    "Chanson",
    "Opera",
    "Chamber Music",
    "Sonata",
    "Symphony",
    "Booty Bass",
    "Primus",
    "Porn Groove",
    "Satire",
    "Slow Jam",
    "Club",
    "Tango",
    "Samba",
    "Folklore",
    "Ballad",
    "Power Ballad",
    "Rhythmic Soul",
    "Freestyle",
    "Duet",
    "Punk Rock",
    "Drum Solo",
    "A Cappella",
    "Euro-House",
    "Dancehall",
    "Goa",
    "Drum & Bass",
    "Club-House",
    "Hardcore Techno",
    "Terror",
    "Indie",
    "Britpop",
    "Worldbeat",
    "Polsk Punk",
    "Beat",
    "Christian Gangsta Rap",
    "Heavy Metal",
    "Black Metal",
    "Crossover",
    "Contemporary Christian",
    "Christian Rock",
    "Merengue",
    "Salsa",
    "Thrash Metal",
    "Anime",
    "Jpop",
    "Synthpop",
    "Abstract",
    "Art Rock",
    "Baroque",
    "Bhangra",
    "Big Beat",
    "Breakbeat",
    "Chillout",
    "Downtempo",
    "Dub",
    "EBM",
    "Eclectic",
    "Electro",
    "Electroclash",
    "Emo",
    "Experimental",
    "Garage",
    "Global",
    "IDM",
    "Illbient",
    "Industro-Goth",
    "Jam Band",
    "Krautrock",
    "Leftfield",
    "Lounge",
    "Math Rock",
    "New Romantic",
    "Nu-Breakz",
    "Post-Punk",
    "Post-Rock",
    "Psytrance",
    "Shoegaze",
    "Space Rock",
    "Trop Rock",
    "World Music",
    "Neoclassical",
    "Audiobook",
    "Audio Theatre",
    "Neue Deutsche Welle",
    "Podcast",
    "Indie Rock",
    "G-Funk",
    "Dubstep",
    "Garage Rock",
    "Psybient",
];

const fix_up_genres: &'static [(&'static str, usize)] = &[
    ("Jazz+Funk", 29),
    ("Folk/Rock", 81),
    ("Bebob", 85),
    ("Avantgarde", 90),
    ("Dance Hall", 125),
    ("Hardcore", 129),
    ("BritPop", 132),
    ("Negerpunk", 133),
];

fn genre(i: usize) -> &'static str {
    if i < genres.len() {
        genres[i]
    } else {
        ""
    }
}

fn genre_index(name: &str) -> usize {
    let i = genres.iter().position(|&x| x == name);

    if let Some(n) = i {
        return n;
    }

    let mut i = 0;
    while i < fix_up_genres.len() {
        if fix_up_genres[i].0 == name {
            return fix_up_genres[i].1;
        }

        i += 1;
    }

    255
}

fn resize(data: &Vec<u8>, new_size: usize) -> Vec<u8> {
    let mut new_data = data.clone();
    if data.len() >= new_size {
        return new_data;
    }

    let old_size = data.len();
    for _ in 0..(new_size - old_size) {
        new_data.push(0);
    }

    new_data
}

fn parse(s: String) -> String {
    String::from(s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let data = vec![b'H', b'e', b'l', b'l', b'o'];
        let data = resize(&data, 8);
        assert_eq!(data, [b'H', b'e', b'l', b'l', b'o', 0, 0, 0]);
        assert_eq!(data.len(), 8);
    }
}
