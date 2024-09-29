use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Global {
    #[serde(default)]
    pub hover: Hover,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hover {
    pub show_next_id: bool,
    pub show_wire_size: bool,
}

impl Default for Hover {
    fn default() -> Self {
        Self {
            show_next_id: true,
            show_wire_size: true,
        }
    }
}
