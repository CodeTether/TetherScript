//! Child-process output forwarding.

use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Sender;

pub(crate) enum Output {
    Line(String),
    Error(String),
}

pub(super) fn forward<R>(stream: R, sender: Sender<Output>, wrap: fn(String) -> Output)
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if sender.send(wrap(line)).is_err() {
                break;
            }
        }
    });
}
