//! Concrete handle storage and type-directed dispatch.

use super::{
    channel, child_process, file, request_body, response_writer, task, tcp_listener, tcp_stream,
    timer,
};

pub(super) enum Payload {
    File(file::Handle),
    ChildProcess(child_process::Handle),
    TcpStream(tcp_stream::Handle),
    TcpListener(tcp_listener::Handle),
    RequestBody(request_body::Handle),
    ResponseWriter(response_writer::Handle),
    Task(task::Handle),
    Timer(timer::Handle),
    Channel(channel::Handle),
}
