use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

pub struct Fixture {
    pub url: String,
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

impl Fixture {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let handle = thread::spawn(move || serve(listener, flag));
        Self { url, stop, handle }
    }

    pub fn stop(self) {
        self.stop.store(true, Ordering::SeqCst);
        self.handle.join().unwrap();
    }
}

fn serve(listener: TcpListener, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _)) => super::fixture_response::reply(&mut stream),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}
