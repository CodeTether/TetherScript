//! Dedicated standard-stream pumps for nonblocking script operations.

use std::io::{Read, Write};
use std::process::ChildStdin;
use std::thread::{self, JoinHandle};

use super::buffer::Buffer;

pub(super) fn input(mut writer: ChildStdin, buffer: Buffer) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Some(bytes) = buffer.pop_blocking(8 * 1024) {
            if let Err(error) = writer.write_all(&bytes) {
                buffer.fail(format!("child_process.stdin: {error}"));
                return;
            }
        }
    })
}

pub(super) fn output<R>(mut reader: R, buffer: Buffer, label: &'static str) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut chunk = [0; 8 * 1024];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => {
                    buffer.close();
                    return;
                }
                Ok(count) => buffer.push_blocking(&chunk[..count]),
                Err(error) => {
                    buffer.fail(format!("{label}: {error}"));
                    return;
                }
            }
        }
    })
}
