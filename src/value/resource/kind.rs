//! Stable runtime tags for owned resources.

/// Identifies the concrete host handle stored by an owned resource.
///
/// Variants cover files, child processes, TCP sockets, streaming HTTP handles,
/// cooperative tasks, timers, and bounded channels.
///
/// # Examples
///
/// ```
/// use tetherscript::value::resource::ResourceKind;
///
/// let kind = ResourceKind::TcpStream;
/// assert_eq!(kind.type_name(), "tcp_stream");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    /// Seekable or streaming host file handle.
    File,
    /// Spawned operating-system child process.
    ChildProcess,
    /// Connected nonblocking TCP byte stream.
    TcpStream,
    /// Bound nonblocking TCP connection listener.
    TcpListener,
    /// Bounded inbound HTTP request bytes.
    RequestBody,
    /// Bounded outbound HTTP response bytes.
    ResponseWriter,
    /// Cooperative task completion handle.
    Task,
    /// Monotonic readiness timer.
    Timer,
    /// Bounded first-in, first-out value channel.
    Channel,
    /// Quota-bound raster rendering surface.
    RenderSurface,
}

impl ResourceKind {
    /// Return the language-visible dynamic type name.
    pub fn type_name(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::ChildProcess => "child_process",
            Self::TcpStream => "tcp_stream",
            Self::TcpListener => "tcp_listener",
            Self::RequestBody => "request_body",
            Self::ResponseWriter => "response_writer",
            Self::Task => "task",
            Self::Timer => "timer",
            Self::Channel => "channel",
            Self::RenderSurface => "render_surface",
        }
    }
}
