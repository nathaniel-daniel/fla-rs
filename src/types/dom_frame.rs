use crate::types::DomShape;

#[derive(Debug, serde::Deserialize)]
pub struct DomFrame {
    pub index: u64,

    #[serde(rename = "keyMode")]
    pub key_mode: u64,

    pub elements: Elements,
}

impl DomFrame {
    pub fn calc_bounding_box(&self) -> Option<euclid::Box2D<f64, euclid::UnknownUnit>> {
        let mut ret = None;
        for bounding_box in self
            .get_shapes()
            .iter()
            .map(|s| s.calc_bounding_box())
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

    pub fn get_shapes(&self) -> &[DomShape] {
        &self.elements.dom_shapes
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Elements {
    #[serde(rename = "DOMShape", default)]
    pub dom_shapes: Vec<DomShape>,
}
