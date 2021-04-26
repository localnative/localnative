use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

pub async fn start_server() -> anyhow::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 2345);
    localnative_core::rpc::server::start_server(&addr).await?;
    Ok(())
}
pub async fn stop_server() -> anyhow::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2345);
    localnative_core::rpc::client::run_stop_server(&addr).await?;
    Ok(())
}