use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Day {
    #[serde(rename = "k")]
    pub time: String,
    #[serde(rename = "v")]
    pub count: i32,
}
