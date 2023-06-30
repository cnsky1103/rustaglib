use std::collections::HashMap;

pub trait Tag {
    fn properties(&self) -> &PropertyMap;

    fn remove_unsupported_properties(&mut self, properties: Vec<String>);

    fn set_properties(&mut self, properties: PropertyMap);

    fn title(&self) -> &Option<String>;
    fn artist(&self) -> &Option<String>;
    fn album(&self) -> &Option<String>;
    fn comment(&self) -> &Option<String>;
    fn genre(&self) -> &Option<String>;
    fn year(&self) -> &Option<u32>;
    fn track(&self) -> &Option<u32>;

    fn set_title(&mut self, title: Option<String>);
    fn set_artist(&mut self, artist: Option<String>);
    fn set_album(&mut self, album: Option<String>);
    fn set_comment(&mut self, comment: Option<String>);
    fn set_genre(&mut self, genre: Option<String>);
    fn set_year(&mut self, year: Option<u32>);
    fn set_track(&mut self, track: Option<u32>);

    fn is_empty(&self) -> bool;
}

pub type PropertyMap = HashMap<String, Vec<String>>;
