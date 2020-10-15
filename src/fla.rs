use crate::{
    types::{
        DomDocument,
        LibraryEntry,
    },
    FlaResult,
};
use std::{
    collections::HashMap,
    io::{
        Read,
        Seek,
    },
};
use zip::ZipArchive;

/// An Fla struct.
#[derive(Debug)]
pub struct Fla {
    pub dom_document: DomDocument,
    pub library: HashMap<String, LibraryEntry>,
}

impl Fla {
    /// Try to get a new fla from a zip file
    pub fn new<R: Read + Seek>(reader: R) -> FlaResult<Self> {
        let mut zip = ZipArchive::new(reader)?;
        let dom_document_file = std::io::BufReader::new(zip.by_name("DOMDocument.xml")?);
        let dom_document: DomDocument = quick_xml::de::from_reader(dom_document_file)?;

        let mut library = HashMap::with_capacity(dom_document.symbols.includes.len());
        for include in dom_document.symbols.includes.iter() {
            let name = include.href.clone();
            let file = std::io::BufReader::new(zip.by_name(&format!("LIBRARY/{}", name))?);
            let entry = LibraryEntry::from_read(&name, file)?;
            library.insert(name, entry);
        }

        Ok(Fla {
            dom_document,
            library,
        })
    }

    /// Get an asset from the library
    pub fn get_library_asset(&self, filename: &str) -> Option<&LibraryEntry> {
        self.library.get(filename)
    }
}
