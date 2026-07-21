//! Public constructors for concrete resource kinds.

use std::path::Path;
use std::time::Duration;

use super::{
    channel, file, payload::Payload, request_body, response_writer, task, tcp_listener, tcp_stream,
    timer, OwnedResource,
};

impl OwnedResource {
    /// Open an owned file in `read`, `write`, `append`, or `read_write` mode.
    ///
    /// # Errors
    ///
    /// Returns a path-qualified error for an invalid mode or failed open.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use tetherscript::value::resource::OwnedResource;
    /// let file = OwnedResource::file(Path::new("Cargo.toml"), "read")?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn file(path: &Path, mode: &str) -> Result<Self, String> {
        file::Handle::open(path, mode).map(|handle| Self::new(Payload::File(handle)))
    }

    /// Connect an owned nonblocking TCP stream within `timeout`.
    ///
    /// # Errors
    ///
    /// Returns address-resolution, timeout, connect, or configuration errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tetherscript::value::resource::OwnedResource;
    /// let stream = OwnedResource::tcp_stream("127.0.0.1", 8080, Duration::from_secs(1))?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn tcp_stream(host: &str, port: u16, timeout: Duration) -> Result<Self, String> {
        tcp_stream::Handle::connect(host, port, timeout)
            .map(|handle| Self::new(Payload::TcpStream(handle)))
    }

    /// Bind an owned nonblocking TCP listener.
    ///
    /// # Errors
    ///
    /// Returns a host-and-port-qualified bind or configuration error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tetherscript::value::resource::OwnedResource;
    /// let listener = OwnedResource::tcp_listener("127.0.0.1", 0)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn tcp_listener(host: &str, port: u16) -> Result<Self, String> {
        tcp_listener::Handle::bind(host, port).map(|handle| Self::new(Payload::TcpListener(handle)))
    }

    /// Create a bounded request body from owned bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the initial body exceeds `capacity`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::value::resource::OwnedResource;
    /// let body = OwnedResource::request_body(b"hello".to_vec(), 5)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn request_body(bytes: Vec<u8>, capacity: usize) -> Result<Self, String> {
        request_body::Handle::new(bytes, capacity)
            .map(|handle| Self::new(Payload::RequestBody(handle)))
    }

    /// Create a bounded response writer.
    ///
    /// # Errors
    ///
    /// Returns an error when `capacity` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::value::resource::OwnedResource;
    /// let writer = OwnedResource::response_writer(1024)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn response_writer(capacity: usize) -> Result<Self, String> {
        response_writer::Handle::new(capacity)
            .map(|handle| Self::new(Payload::ResponseWriter(handle)))
    }

    /// Create a pending cooperative task completion handle.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::value::resource::{OwnedResource, ResourceKind};
    /// assert_eq!(OwnedResource::task().kind(), ResourceKind::Task);
    /// ```
    pub fn task() -> Self {
        Self::new(Payload::Task(task::Handle::pending()))
    }

    /// Create a monotonic timer that becomes ready after `duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use tetherscript::value::resource::OwnedResource;
    /// let timer = OwnedResource::timer(Duration::ZERO);
    /// assert!(!timer.is_closed());
    /// ```
    pub fn timer(duration: Duration) -> Self {
        Self::new(Payload::Timer(timer::Handle::after(duration)))
    }

    /// Create a bounded channel.
    ///
    /// # Errors
    ///
    /// Returns an error when `capacity` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::value::resource::OwnedResource;
    /// let channel = OwnedResource::channel(8)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn channel(capacity: usize) -> Result<Self, String> {
        channel::Handle::bounded(capacity).map(|handle| Self::new(Payload::Channel(handle)))
    }
}
