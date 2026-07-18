use std::io::{Read, Write};

pub fn reply(stream: &mut std::net::TcpStream) {
    let mut request = [0; 1024];
    let _ = stream.read(&mut request);
    let body = "<button id='change'>ready</button><script>document.querySelector('#change').addEventListener('click',function(event){this.textContent='clicked';window.lastTrusted=event.isTrusted;});</script>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
}
