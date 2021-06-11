use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Day {
    #[serde(rename = "k")]
    pub day: String,
    #[serde(rename = "v")]
    pub count: i32,
}
