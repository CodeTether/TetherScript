use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

#[test]
fn cli_loads_provider_from_codetether_style_vault_secret() {
    let (addr, handle) = spawn_vault();
    let out = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args([
            "run",
            "--grant-provider-vault",
            "openai",
            "examples/provider_vault_describe.tether",
        ])
        .env("VAULT_ADDR", format!("http://{addr}"))
        .env("VAULT_TOKEN", "root-token")
        .env("VAULT_MOUNT", "secret")
        .env("VAULT_SECRETS_PATH", "codetether/providers")
        .output()
        .expect("tetherscript should run");
    let request = handle.join().expect("vault thread should finish");
    assert!(
        request.contains("GET /v1/secret/data/codetether/providers/openai "),
        "request was:\n{request}"
    );
    assert!(
        request.contains("X-Vault-Token: root-token"),
        "request was:\n{request}"
    );
    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "https://api.openai.com\n/v1/chat/completions\nAuthorization,OpenAI-Organization\n"
    );
}

fn spawn_vault() -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0u8; 512];
        loop {
            let n = stream.read(&mut buf).unwrap();
            bytes.extend_from_slice(&buf[..n]);
            if n == 0 || bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let request = String::from_utf8_lossy(&bytes).into_owned();
        let body = r#"{"data":{"data":{"api_key":"sk-test","base_url":"https://api.openai.com/v1","organization":"org-test"}}}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
        request
    });
    (addr, handle)
}
