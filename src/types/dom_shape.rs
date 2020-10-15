pub mod edge;

pub use self::edge::{
    Edge,
    EdgeDefinitionCommand,
    SelectionMask,
};

#[derive(Debug, serde::Deserialize)]
pub struct DomShape {
    pub selected: Option<bool>,

    #[serde(rename = "isFloating")]
    pub is_floating: Option<bool>,

    #[serde(rename = "matrix")]
    pub matrices: Option<Matrices>,
    pub fills: Option<Fills>,
    pub strokes: Option<Strokes>,
    pub edges: Edges,
}

impl DomShape {
    pub fn get_fill_style(&self, index: u64) -> Option<&FillStyle> {
        self.fills
            .as_ref()?
            .fill_styles
            .iter()
            .find(|el| el.index == Some(index))
    }

    pub fn get_stroke_style(&self, index: u64) -> Option<&StrokeStyle> {
        self.strokes
            .as_ref()?
            .stroke_styles
            .iter()
            .find(|el| el.index == Some(index))
    }

    pub fn get_edges(&self) -> &[Edge] {
        &self.edges.edges
    }

    pub fn calc_bounding_box(&self) -> Option<euclid::Box2D<f64, euclid::UnknownUnit>> {
        let mut min_x: Option<f64> = None;
        let mut min_y: Option<f64> = None;
        let mut max_x: Option<f64> = None;
        let mut max_y: Option<f64> = None;

        for edge in self.get_edges() {
            for cmd in edge
                .edges
                .iter()
                .map(|edge_def| edge_def.commands.iter())
                .flatten()
            {
                match cmd {
                    EdgeDefinitionCommand::MoveTo(x, y)
                    | EdgeDefinitionCommand::LineTo(x, y)
                    | EdgeDefinitionCommand::CurveTo(x, y, _, _) => {
                        let min_x = min_x.get_or_insert(*x);
                        let min_y = min_y.get_or_insert(*y);
                        let max_x = max_x.get_or_insert(*x);
                        let max_y = max_y.get_or_insert(*y);
                        *min_x = min_x.min(*x);
                        *min_y = min_y.min(*y);
                        *max_x = max_x.max(*x);
                        *max_y = max_y.max(*y);
                    }
                    EdgeDefinitionCommand::Selection(_mask) => {
                        // Nothing...
                    }
                }
            }
        }

        Some(euclid::Box2D::new(
            euclid::Point2D::new(min_x?, min_y?),
            euclid::Point2D::new(max_x?, max_y?),
        ))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Matrices {
    #[serde(rename = "Matrix", default)]
    pub matrix: Vec<Matrix>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Matrix {
    pub a: Option<f64>,
    pub d: Option<f64>,
    pub tx: Option<f64>,
    pub ty: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Fills {
    #[serde(rename = "FillStyle", default)]
    pub fill_styles: Vec<FillStyle>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FillStyle {
    pub index: Option<u64>,

    #[serde(rename = "SolidColor")]
    pub solid_color: Option<SolidColor>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SolidColor {
    pub color: Option<String>,
}

impl SolidColor {
    pub fn get_rgb(&self) -> Option<(u8, u8, u8)> {
        let color_str = match self.color.as_ref() {
            Some(color_str) => color_str,
            None => return Some((0, 0, 0)), // Null is black apparently
        };

        let r = u8::from_str_radix(&color_str.get(1..3)?, 16).ok()?;
        let g = u8::from_str_radix(&color_str.get(3..5)?, 16).ok()?;
        let b = u8::from_str_radix(&color_str.get(5..7)?, 16).ok()?;

        Some((r, g, b))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Strokes {
    #[serde(rename = "StrokeStyle", default)]
    pub stroke_styles: Vec<StrokeStyle>,
}

#[derive(Debug, serde::Deserialize)]
pub struct StrokeStyle {
    pub index: Option<u64>,

    #[serde(rename = "SolidStroke")]
    pub solid_stroke: SolidStroke,
}

#[derive(Debug, serde::Deserialize)]
pub struct SolidStroke {
    pub fill: Fill,
}

#[derive(Debug, serde::Deserialize)]
pub struct Fill {
    pub solid_color: Option<SolidColor>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Edges {
    #[serde(rename = "Edge", default)]
    pub edges: Vec<Edge>,
}
