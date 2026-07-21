//! Supervised child processes with bounded standard streams.

mod buffer;
mod buffer_read;
mod buffer_write;
mod call;
mod lifecycle;
mod pumps;
mod spawn;
mod status;
mod streams;
mod wait;

use std::process::Child;
use std::thread::JoinHandle;

use buffer::Buffer;

pub(super) struct Handle {
    child: Child,
    stdin: Buffer,
    stdout: Buffer,
    stderr: Buffer,
    workers: Vec<JoinHandle<()>>,
}

impl Handle {
    pub(super) fn spawn(command: &str, args: &[String], capacity: usize) -> Result<Self, String> {
        let stdin = Buffer::new(capacity, "child_process.stdin")?;
        let stdout = Buffer::new(capacity, "child_process.stdout")?;
        let stderr = Buffer::new(capacity, "child_process.stderr")?;
        let mut child = spawn::child(command, args)?;
        let input = child.stdin.take().expect("piped child stdin must exist");
        let output = child.stdout.take().expect("piped child stdout must exist");
        let errors = child.stderr.take().expect("piped child stderr must exist");
        let workers = vec![
            pumps::input(input, stdin.clone()),
            pumps::output(output, stdout.clone(), "child_process.stdout"),
            pumps::output(errors, stderr.clone(), "child_process.stderr"),
        ];
        Ok(Self {
            child,
            stdin,
            stdout,
            stderr,
            workers,
        })
    }
}
