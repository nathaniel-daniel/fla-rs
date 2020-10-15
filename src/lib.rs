pub mod types;

use crate::types::{
    dom_shape::{
        EdgeDefinitionCommand,
        SelectionMask,
    },
    DomDocument,
    DomSymbol,
    LibraryEntry,
};
use std::{
    collections::HashMap,
    io::{
        Read,
        Seek,
    },
};
use zip::ZipArchive;

/// Result type
pub type FlaResult<T> = Result<T, FlaError>;

/// Error Type
#[derive(Debug, thiserror::Error)]
pub enum FlaError {
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

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

/// Render a DomSymbol.
pub fn render_dom_symbol(symbol: &DomSymbol, scale: f64) -> Option<Vec<raqote::DrawTarget>> {
    let bounding_box = symbol.calc_bounding_box()?;
    let draw_target_width = (bounding_box.width() * scale) as i32;
    let draw_target_height = (bounding_box.height() * scale) as i32;

    let num_frames = symbol.num_frames();
    let mut frames = Vec::with_capacity(num_frames);

    let mut layers: Vec<_> = symbol
        .get_layers()
        .iter()
        .map(|layer| layer.get_frames().iter().cycle())
        .collect();

    let transform = raqote::Transform::create_translation(
        (-bounding_box.min.x * scale) as f32,
        (-bounding_box.min.y * scale) as f32,
    )
    .pre_scale(scale as f32, scale as f32);

    let draw_options = raqote::DrawOptions::new();

    for _ in 0..num_frames {
        let mut target = raqote::DrawTarget::new(draw_target_width, draw_target_height);

        for frame in layers
            .iter_mut()
            .map(|layer_frame_iter| layer_frame_iter.next().unwrap())
        {
            for shape in frame.get_shapes() {
                for edge in shape.get_edges().iter() {
                    let mut pb = raqote::PathBuilder::new();
                    let mut last_selection_mask = None;

                    for cmd in edge
                        .get_edge_definition_commands()
                        .iter()
                        .map(|cmds| cmds.iter())
                        .flatten()
                    {
                        match cmd {
                            EdgeDefinitionCommand::MoveTo(x, y) => {
                                let x = *x as f32;
                                let y = *y as f32;

                                if last_selection_mask.is_none() {
                                    pb.move_to(x, y);
                                } else {
                                    pb.line_to(x, y);
                                }
                            }
                            EdgeDefinitionCommand::Selection(selection_mask) => {
                                if let Some(_last_selection_mask) = last_selection_mask {
                                    return None; // TODO: Write to target and instantiate new path builder.
                                } else {
                                    last_selection_mask = Some(selection_mask);
                                }
                            }
                            EdgeDefinitionCommand::CurveTo(x, y, ex, ey) => {
                                let x = *x as f32;
                                let y = *y as f32;
                                let ex = *ex as f32;
                                let ey = *ey as f32;
                                pb.quad_to(x, y, ex, ey);
                            }
                            _ => return None, // Unsupported Command
                        }
                    }
                    pb.close();

                    if let Some(selection_mask) = last_selection_mask {
                        // Ignore multi-flags.
                        // TODO: handle multi-flags.

                        let path = pb.finish().transform(&transform);

                        if selection_mask.contains(SelectionMask::FILLSTYLE0) {
                            return None; // Unsupported
                        } else if selection_mask.contains(SelectionMask::FILLSTYLE1) {
                            let fill_style_1_index = edge.fill_style_1?;
                            let fill_style_1 = shape.get_fill_style(fill_style_1_index)?;

                            // Only support solid color for now
                            let color = fill_style_1.solid_color.as_ref()?.get_rgb()?;

                            let color = raqote::SolidSource {
                                r: color.0,
                                g: color.1,
                                b: color.2,
                                a: 0xFF,
                            };
                            target.fill(&path, &raqote::Source::Solid(color), &draw_options);
                        } else if selection_mask.contains(SelectionMask::STROKE) {
                            return None; // Unsupported
                        }
                    }
                }
            }
        }

        frames.push(target);
    }

    Some(frames)
}
