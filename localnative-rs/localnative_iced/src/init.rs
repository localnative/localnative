// use std::{env, path::Path};

use serde::Serialize;

// use crate::config::app_dir;

#[derive(Debug, Default, Serialize)]
pub struct AppHost {
    name: String,
    description: String,
    path: String,
    #[serde(rename = "type")]
    tp: String,
    allowed_extensions: Vec<String>,
}
// impl AppHost {
//     pub fn path() -> String {
//         let version = "0.4.2";
//         let platform = if cfg!(windows) {
//             ".exe"
//         } else if cfg!(macos) {
//             // TODO: 需要实际去检查是否是这个扩展名
//             ""
//         } else {
//             // TODO：需要后续的处理
//             ""
//         };
//         let mut path = app_dir().join("bin");
//         path.push(String::from("localnative-web-ext-host-") + version + platform);
//         if path.exists() {
//             println!("是的，存在这个文件。。");
//         }
//         path.to_str().unwrap().to_owned()
//     }
//     pub fn firefox() -> Self {
//         Self {
//             name: "app.localnative".to_owned(),
//             description: "Local Native Host".to_owned(),
//             path: Self::path(),
//             tp: "stdio".to_owned(),
//             allowed_extensions: vec!["localnative@example.org".to_owned()],
//         }
//     }
//     pub fn chrome() -> Self {
//         Self {
//             name: "app.localnative".to_owned(),
//             description: "Local Native Host".to_owned(),
//             path: Self::path(),
//             tp: "stdio".to_owned(),
//             allowed_extensions: vec![
//                 "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/".to_owned()
//             ],
//         }
//     }
//     pub fn json(&self) -> anyhow::Result<String> {
//         Ok(serde_json::to_string(self)?)
//     }
// }

// pub async fn init_app_host() -> anyhow::Result<()> {

//     Ok(())
// }
