use crate::types::{
    DomLayer,
    DomTimeline,
};

#[derive(Debug, serde::Deserialize)]
pub struct DomSymbol {
    pub name: String,

    #[serde(rename = "itemID")]
    pub item_id: String,

    #[serde(rename = "linkageExportForAS")]
    pub linkage_export_for_as: Option<bool>,

    #[serde(rename = "linkageClassName")]
    pub linkage_class_name: Option<String>,

    #[serde(rename = "lastModified")]
    pub last_modified: String,

    pub timeline: Timeline,
}

impl DomSymbol {
    pub fn get_layers(&self) -> &[DomLayer] {
        &self.timeline.dom_timeline.get_layers()
    }

    pub fn get_layer(&self, index: usize) -> Option<&DomLayer> {
        self.get_layers().get(index)
    }

    pub fn calc_bounding_box(&self) -> Option<euclid::Box2D<f64, euclid::UnknownUnit>> {
        let mut ret = None;
        for bounding_box in self
            .get_layers()
            .iter()
            .map(|l| l.calc_bounding_box())
            .flatten()
        {
            let bb = ret.get_or_insert(bounding_box);
            bb.min.x = bb.min.x.min(bounding_box.min.x);
            bb.min.y = bb.min.y.min(bounding_box.min.y);
            bb.max.x = bb.max.x.max(bounding_box.max.x);
            bb.max.y = bb.max.y.max(bounding_box.max.y);
        }

        ret
    }

    pub fn num_frames(&self) -> usize {
        let mut num_frames = 0;
        for layer in self.get_layers().iter() {
            num_frames = num_frames.max(layer.num_frames());
        }

        num_frames
    }
}

#[cfg(feature = "render-raqote")]
#[derive(Debug, thiserror::Error)]
pub enum DomSymbolRenderError {
    #[error("Could not determine a bounding box")]
    NoBoundingBox,

    #[error("Missing FillStyle{0}")]
    MissingFillStyleIndex(usize),

    #[error("Missing FillStyle {0}")]
    MissingFillStyle(u64),

    #[error("Missing Fill Stroke Style Index")]
    MissingStrokeStyleIndex,

    #[error("Missing StrokeStyle {0}")]
    MissingStrokeStyle(u64),

    #[error("Missing Color")]
    MissingColor,

    #[error("InvalidRbg")]
    InvalidRbg,

    #[error("unsupported: {0}")]
    Unsupported(&'static str),
}

#[cfg(feature = "render-raqote")]
impl DomSymbol {
    /// Render a DomSymbol.
    pub fn render_raqote(
        &self,
        scale: f64,
        padding: f64,
    ) -> Result<Vec<raqote::DrawTarget>, DomSymbolRenderError> {
        let bounding_box = self
            .calc_bounding_box()
            .ok_or(DomSymbolRenderError::NoBoundingBox)?;
        let draw_target_width = (bounding_box.width() * scale) as i32 + padding as i32;
        let draw_target_height = (bounding_box.height() * scale) as i32 + padding as i32;

        let num_frames = symbol.num_frames();
        let mut frames = Vec::with_capacity(num_frames);

        let mut layers: Vec<_> = symbol
            .get_layers()
            .iter()
            .map(|layer| layer.get_frames().iter().cycle())
            .collect();

        let transform = raqote::Transform::create_translation(
            (-bounding_box.min.x * scale) as f32 + (padding / 2.0) as f32,
            (-bounding_box.min.y * scale) as f32 + (padding / 2.0) as f32,
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
                                EdgeDefinitionCommand::LineTo(x, y) => {
                                    let x = *x as f32;
                                    let y = *y as f32;
                                    pb.line_to(x, y);
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
                            }
                        }
                        pb.close();

                        if let Some(selection_mask) = last_selection_mask {
                            let path = pb.finish().transform(&transform);

                            if selection_mask.contains(SelectionMask::FILLSTYLE0) {
                                return Err(DomSymbolRenderError::Unsupported("FILLSTYLE0"));
                            }

                            if selection_mask.contains(SelectionMask::FILLSTYLE1) {
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
                            }

                            if selection_mask.contains(SelectionMask::STROKE) {
                                let stroke_style_index = edge
                                    .stroke_style
                                    .ok_or(DomSymbolRenderError::MissingStrokeStyleIndex)?;
                                let stroke_style = shape
                                    .get_stroke_style(stroke_style_index)
                                    .ok_or(DomSymbolRenderError::MissingStrokeStyle(
                                        stroke_style_index,
                                    ))?;

                                // Only support solid color for now
                                let color = stroke_style
                                    .solid_stroke
                                    .fill
                                    .solid_color
                                    .as_ref()
                                    .map(|solid_color| {
                                        solid_color
                                            .get_rgb()
                                            .ok_or(DomSymbolRenderError::InvalidRbg)
                                    })
                                    .unwrap_or(Ok((0, 0, 0)))?;

                                let color = raqote::SolidSource {
                                    r: color.0,
                                    g: color.1,
                                    b: color.2,
                                    a: 0xFF,
                                };

                                let mut stroke_style = raqote::StrokeStyle::default();
                                stroke_style.cap = raqote::LineCap::Round;
                                stroke_style.join = raqote::LineJoin::Round;
                                stroke_style.width = 20.0 * scale as f32;

                                target.stroke(
                                    &path,
                                    &raqote::Source::Solid(color),
                                    &stroke_style,
                                    &draw_options,
                                );
                            }
                        }
                    }
                }
            }

            frames.push(target);
        }

        Ok(frames)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Timeline {
    #[serde(rename = "DOMTimeline")]
    pub dom_timeline: DomTimeline,
}
