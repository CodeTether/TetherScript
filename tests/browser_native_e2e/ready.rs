use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn wait(address: std::net::SocketAddr) {
    let body = r#"{"action":"health"}"#;
    for _ in 0..100 {
        if post(address, body).is_ok_and(|response| response.contains(r#""ok":true"#)) {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("native browser host did not become ready");
}

pub fn post(address: std::net::SocketAddr, body: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(address).map_err(|error| error.to_string())?;
    let request = format!(
        "POST /browser HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| error.to_string())?;
    Ok(response)
}
