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

    pub fn finish_after_script(&mut self) {
        for _ in 0..100 {
            if let Some(status) = self.child.try_wait().expect("poll native browser host") {
                assert!(status.success(), "native browser host failed: {status}");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        self.stop();
        panic!("native browser script did not stop its host");
    }
}
