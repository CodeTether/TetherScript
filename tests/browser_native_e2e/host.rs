use std::net::TcpListener;
use std::process::{Child, Command, Stdio};

pub struct NativeHost {
    pub endpoint: String,
    address: std::net::SocketAddr,
    child: Child,
}

impl NativeHost {
    pub fn start() -> Self {
        let probe = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = probe.local_addr().unwrap();
        drop(probe);
        let child = Command::new(env!("CARGO_BIN_EXE_tetherscript-browser-host"))
            .arg(address.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start native browser host");
        let host = Self {
            endpoint: format!("http://{address}/browser"),
            address,
            child,
        };
        super::ready::wait(address);
        host
    }

    pub fn finish(&mut self) {
        let status = self.child.wait().expect("wait for native browser host");
        assert!(status.success(), "native browser host failed: {status}");
    }

    pub fn stop(&mut self) {
        super::ready::post(self.address, r#"{"action":"stop"}"#).unwrap();
        self.finish();
    }
}
