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
            bb.min.x = bb.min.y.min(bounding_box.min.x);
            bb.min.y = bb.min.y.min(bounding_box.min.y);
            bb.max.x = bb.max.y.max(bounding_box.max.x);
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

#[derive(Debug, serde::Deserialize)]
pub struct Timeline {
    #[serde(rename = "DOMTimeline")]
    pub dom_timeline: DomTimeline,
}
