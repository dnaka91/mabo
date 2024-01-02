use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Global {
    pub max_number_of_problems: u32,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            max_number_of_problems: 100,
        }
    }
}
