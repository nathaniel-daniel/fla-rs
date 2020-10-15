use crate::types::DomLayer;

#[derive(Debug, serde::Deserialize)]
pub struct DomTimeline {
    pub name: String,
    pub guides: Option<String>,
    pub layers: Layers,
}

impl DomTimeline {
    pub fn get_layers(&self) -> &[DomLayer] {
        &self.layers.dom_layers
    }

    pub fn get_layer(&self, index: usize) -> Option<&DomLayer> {
        self.get_layers().get(index)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Layers {
    #[serde(rename = "DOMLayer", default)]
    pub dom_layers: Vec<DomLayer>,
}
