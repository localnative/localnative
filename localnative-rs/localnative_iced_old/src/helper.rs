use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use localnative_core::rpc::server::Stop;
use native_dialog::FileDialog;

use crate::tr;

// use iced::{button, text_input, tooltip, Element, PickList, Row, Text, TextInput};

// pub fn tip_button<E,M,'a>(state:&'a mut button::State,content:E,msg:M,button_style:impl Into<Renderer::Style>,tip:String) -> Element<'a,M>
// where
//     E: Into<Element<'a, M>>
// {
//     tooltip::Tooltip::new(
//         button::Button::new(state,content)
//         .style(crate::style::symbol::Symbol)
//         .on_press(msg),
//         tip,
//         tooltip::Position::FollowCursor
//     )
//     .style(crate::style::symbol::Symbol)
//     .into()
// }
pub fn get_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => Some(addr.ip().to_string() + ":2345"),
        Err(_) => None,
    }
}

pub async fn start_server() -> anyhow::Result<Stop> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 2345);
    localnative_core::rpc::server::iced_start_server(addr)
        .await
        .map_err(|e| anyhow::anyhow!("server error:{:?}", e))
}
pub async fn stop_server(stop: Stop) -> anyhow::Result<()> {
    let res = stop.await?;
    drop(res);
    Ok(())
}
pub async fn client_sync_from_server(addr: SocketAddr) -> anyhow::Result<()> {
    localnative_core::rpc::client::run_sync_from_server(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("sync from server fail:{:?}", e))?;
    Ok(())
}
pub async fn client_sync_to_server(addr: SocketAddr) -> anyhow::Result<()> {
    localnative_core::rpc::client::run_sync_to_server(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("sync to server fail:{:?}", e))?;
    Ok(())
}
pub async fn get_sync_file_path() -> anyhow::Result<PathBuf> {
    sync_get_sync_file_path()
}
pub fn sync_get_sync_file_path() -> anyhow::Result<PathBuf> {
    let file = FileDialog::new()
        .set_location(&crate::setting_view::app_dir())
        .add_filter(&tr!("sync-file"), &["sqlite3"])
        .show_open_single_file()?;
    match file {
        Some(path) => Ok(path),
        None => Err(anyhow::anyhow!("get file path fial.")),
    }
}
pub async fn sync_via_file(res: anyhow::Result<PathBuf>) -> anyhow::Result<()> {
    if let Ok(path) = res {
        let res = tokio::task::spawn_blocking(move || {
            let uri = path.to_str().unwrap();
            let conn = localnative_core::exe::get_sqlite_connection();
            localnative_core::cmd::sync_via_attach(&conn, uri)
        })
        .await?;
        log::info!("sync via file res:{:?}", res);
    } else {
        log::warn!("get sync via file path fail.");
    }
    Ok(())
}
