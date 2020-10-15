pub mod fla;
pub mod types;

pub use crate::fla::Fla;
use crate::types::{
    dom_shape::{
        EdgeDefinitionCommand,
        SelectionMask,
    },
    DomSymbol,
};

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

#[derive(Debug, thiserror::Error)]
pub enum DomSymbolRenderError {
    #[error("Could not determine a bounding box")]
    NoBoundingBox,

    #[error("Missing FillStyle{0}")]
    MissingFillStyleIndex(usize),

    #[error("Missing FillStyle {0}")]
    MissingFillStyle(u64),

    #[error("Missing Color")]
    MissingColor,

    #[error("InvalidRbg")]
    InvalidRbg,

    #[error("unsupported: {0}")]
    Unsupported(&'static str),
}

/// Render a DomSymbol.
pub fn render_dom_symbol(
    symbol: &DomSymbol,
    scale: f64,
) -> Result<Vec<raqote::DrawTarget>, DomSymbolRenderError> {
    let bounding_box = symbol
        .calc_bounding_box()
        .ok_or(DomSymbolRenderError::NoBoundingBox)?;
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
                                    // TODO: Write to target and instantiate new path builder.
                                    return Err(DomSymbolRenderError::Unsupported(
                                        "SelectionMask overwrite",
                                    ));
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
                            _ => {
                                return Err(DomSymbolRenderError::Unsupported(
                                    "EdgeDefinitionCommand",
                                ))
                            }
                        }
                    }
                    pb.close();

                    if let Some(selection_mask) = last_selection_mask {
                        // Ignore multi-flags.
                        // TODO: handle multi-flags.

                        let path = pb.finish().transform(&transform);

                        if selection_mask.contains(SelectionMask::FILLSTYLE0) {
                            return Err(DomSymbolRenderError::Unsupported("FILLSTYLE0"));
                        } else if selection_mask.contains(SelectionMask::FILLSTYLE1) {
                            let fill_style_1_index = edge
                                .fill_style_1
                                .ok_or(DomSymbolRenderError::MissingFillStyleIndex(1))?;
                            let fill_style_1 = shape.get_fill_style(fill_style_1_index).ok_or(
                                DomSymbolRenderError::MissingFillStyle(fill_style_1_index),
                            )?;

                            // Only support solid color for now
                            let color = fill_style_1
                                .solid_color
                                .as_ref()
                                .ok_or(DomSymbolRenderError::MissingColor)?
                                .get_rgb()
                                .ok_or(DomSymbolRenderError::InvalidRbg)?;

                            let color = raqote::SolidSource {
                                r: color.0,
                                g: color.1,
                                b: color.2,
                                a: 0xFF,
                            };
                            target.fill(&path, &raqote::Source::Solid(color), &draw_options);
                        } else if selection_mask.contains(SelectionMask::STROKE) {
                            return Err(DomSymbolRenderError::Unsupported("STROKE"));
                        }
                    }
                }
            }
        }

        frames.push(target);
    }

    Ok(frames)
}
