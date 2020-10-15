pub mod dom_document;
pub mod dom_frame;
pub mod dom_layer;
pub mod dom_shape;
pub mod dom_symbol;
pub mod dom_timeline;

pub use self::{
    dom_document::DomDocument,
    dom_frame::DomFrame,
    dom_layer::DomLayer,
    dom_shape::DomShape,
    dom_symbol::DomSymbol,
    dom_timeline::DomTimeline,
};
use crate::FlaResult;
use std::{
    io::{
        BufRead,
        Read,
    },
    path::Path,
};

#[derive(Debug, serde::Deserialize)]
pub struct DomFontItem {
    pub name: String,

    #[serde(rename = "itemID")]
    pub item_id: String,

    pub font: String,
    pub size: u64,
    pub id: u64,

    #[serde(rename = "sourceLastImported")]
    pub source_last_imported: String,

    #[serde(rename = "embedRanges")]
    pub embed_ranges: String,
}

#[derive(Debug)]
pub enum LibraryEntry {
    Xml(DomSymbol),
    Unknown(Vec<u8>),
}

impl LibraryEntry {
    pub fn from_read<R: Read + BufRead>(name: &str, mut reader: R) -> FlaResult<Self> {
        match Path::new(name)
            .extension()
            .map(|s| s.to_string_lossy())
            .as_deref()
        {
            Some("xml") => {
                let dom_symbol: DomSymbol = quick_xml::de::from_reader(reader)?;

                Ok(Self::Xml(dom_symbol))
            }
            Some(_) | None => {
                let mut entry = Vec::new();
                std::io::copy(&mut reader, &mut entry)?;

                Ok(Self::Unknown(entry))
            }
        }
    }

    pub fn as_xml(&self) -> Option<&DomSymbol> {
        match self {
            Self::Xml(x) => Some(x),
            _ => None,
        }
    }
}
