#[tauri::command]
pub fn input(input: String) -> String {
    localnative_core::exe::run(&input)
}

#[tauri::command]
pub fn local_ip() -> Result<String, String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|err| {
        println!("bind local udp failed: {}", err);
        String::from("bind local udp failed")
    })?;

    socket.connect("255.255.255.255:80").map_err(|err| {
        println!("local udp connect failed: {}", err);
        String::from("local udp connect failed")
    })?;

    let addr = socket.local_addr().map_err(|err| {
        println!("get local udp addr failed: {}", err);
        String::from("get local udp addr failed")
    })?;

    Ok(addr.ip().to_string())
}
