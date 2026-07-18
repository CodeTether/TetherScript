use std::io::{Read, Write};

pub fn reply(stream: &mut std::net::TcpStream) {
    let mut request = [0; 1024];
    let _ = stream.read(&mut request);
    let body = "<button id='change'>ready</button><input id='entry'><div style='height:1200px'></div><button id='below'>below</button><script>document.querySelector('#change').addEventListener('click',function(event){this.textContent='clicked';window.lastTrusted=event.isTrusted;});let n=document.querySelector('#entry');n.addEventListener('focus',function(){this.setAttribute('data-focus','focused');});n.addEventListener('blur',function(){this.setAttribute('data-focus','blurred');});n.addEventListener('keydown',function(e){this.setAttribute('data-key-events',(this.getAttribute('data-key-events')||'')+'d'+e.key);});n.addEventListener('input',function(){this.setAttribute('data-key-events',this.getAttribute('data-key-events')+'i');});n.addEventListener('keyup',function(e){this.setAttribute('data-key-events',this.getAttribute('data-key-events')+'u'+e.key);});</script>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
}
