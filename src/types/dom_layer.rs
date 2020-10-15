pub use crate::types::DomFrame;

#[derive(Debug, serde::Deserialize)]
pub struct DomLayer {
    pub name: String,
    pub color: String,
    pub current: Option<bool>,

    #[serde(rename = "isSelected")]
    pub is_selected: Option<bool>,

    #[serde(rename = "autoNamed")]
    pub auto_named: Option<bool>,

    pub frames: Frames,
}

impl DomLayer {
    pub fn get_frames(&self) -> &[DomFrame] {
        &self.frames.dom_frames
    }

    pub fn calc_bounding_box(&self) -> Option<euclid::Box2D<f64, euclid::UnknownUnit>> {
        let mut ret = None;
        for bounding_box in self
            .get_frames()
            .iter()
            .map(|f| f.calc_bounding_box())
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
        self.get_frames().len()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Frames {
    #[serde(rename = "DOMFrame", default)]
    pub dom_frames: Vec<DomFrame>,
}
