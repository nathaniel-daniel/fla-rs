use crate::types::{
    DomFontItem,
    DomTimeline,
};

#[derive(Debug, serde::Deserialize)]
pub struct DomDocument {
    #[serde(rename = "backgroundColor")]
    pub background_color: String,

    pub width: u64,
    pub height: u64,

    #[serde(rename = "frameRate")]
    pub frame_rate: u64,

    #[serde(rename = "currentTimeline")]
    pub current_timeline: u64,

    #[serde(rename = "xflVersion")]
    pub xfl_version: String,

    #[serde(rename = "creatorInfo")]
    pub creator_info: String,
    pub platform: String,

    #[serde(rename = "versionInfo")]
    pub version_info: String,

    #[serde(rename = "majorVersion")]
    pub major_version: u64,

    #[serde(rename = "minorVersion")]
    pub minor_version: u64,

    #[serde(rename = "buildNumber")]
    pub build_number: u64,

    #[serde(rename = "rulerVisible")]
    pub ruler_visible: bool,

    #[serde(rename = "viewAngle3D")]
    pub view_angle_3d: f64,

    #[serde(rename = "nextSceneIdentifier")]
    pub next_scene_identifier: u64,

    #[serde(rename = "playOptionsPlayLoop")]
    pub play_options_play_loop: bool,

    #[serde(rename = "playOptionsPlayPages")]
    pub play_options_play_pages: bool,

    #[serde(rename = "playOptionsPlayFrameActions")]
    pub play_options_play_frame_actions: bool,

    #[serde(rename = "filetypeGUID")]
    pub filetype_guid: String,

    #[serde(rename = "fileGUID")]
    pub file_guid: String,

    pub fonts: Fonts,
    pub media: Media,
    pub symbols: Symbols,
    pub timelines: Timelines,
    pub scripts: Scripts,

    #[serde(rename = "persistentData")]
    pub persistent_data: PersistentData,

    #[serde(rename = "PrinterSettings")]
    pub printer_settings: PrinterSettings,

    #[serde(rename = "publishHistory")]
    pub publish_history: PublishHistory,

    #[serde(rename = "swcCache")]
    pub swc_cache: SwcCache,
}

#[derive(Debug, serde::Deserialize)]
pub struct Fonts {
    #[serde(rename = "DOMFontItem", default)]
    pub dom_font_items: Vec<DomFontItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Media {}

#[derive(Debug, serde::Deserialize)]
pub struct Symbols {
    #[serde(rename = "Include", default)]
    pub includes: Vec<SymbolInclude>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SymbolInclude {
    pub href: String,

    #[serde(rename = "itemIcon")]
    pub item_icon: Option<u64>,

    #[serde(rename = "loadImmediate")]
    pub load_immediate: Option<bool>,

    #[serde(rename = "itemID")]
    pub item_id: String,

    #[serde(rename = "lastModified")]
    pub last_modified: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Timelines {
    #[serde(rename = "DomTimeline", default)]
    pub dom_timelines: Vec<DomTimeline>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Scripts {}

#[derive(Debug, serde::Deserialize)]
pub struct PersistentData {}

#[derive(Debug, serde::Deserialize)]
pub struct PrinterSettings {}

#[derive(Debug, serde::Deserialize)]
pub struct PublishHistory {
    #[serde(rename = "PublishItem", default)]
    pub publish_items: Vec<PublishItem>,
}

/// <PublishItem publishSize="12223435" publishTime="1520639252" publishDebug="true"/>
#[derive(Debug, serde::Deserialize)]
pub struct PublishItem {
    #[serde(rename = "publishSize")]
    pub publish_size: u64,

    #[serde(rename = "publishTime")]
    pub publish_time: u64,

    #[serde(rename = "publishDebug")]
    pub publish_debug: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SwcCache {}
