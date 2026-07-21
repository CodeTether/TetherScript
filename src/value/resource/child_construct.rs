//! Public constructors for supervised child processes.

use super::{child_process, payload::Payload, OwnedResource};

const DEFAULT_STREAM_CAPACITY: usize = 64 * 1024;

impl OwnedResource {
    /// Spawn a supervised process with bounded piped standard streams.
    ///
    /// # Errors
    /// Returns a command-qualified spawn error or invalid-capacity error.
    ///
    /// # Examples
    /// ```no_run
    /// use tetherscript::value::resource::OwnedResource;
    /// let child = OwnedResource::child_process("worker", &["--once".into()])?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn child_process(command: &str, args: &[String]) -> Result<Self, String> {
        Self::child_process_bounded(command, args, DEFAULT_STREAM_CAPACITY)
    }

    /// Spawn a supervised process with a per-stream byte capacity.
    ///
    /// # Errors
    /// Returns a command-qualified spawn error or rejects a zero capacity.
    ///
    /// # Examples
    /// ```no_run
    /// use tetherscript::value::resource::OwnedResource;
    /// let child = OwnedResource::child_process_bounded("worker", &[], 4096)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn child_process_bounded(
        command: &str,
        args: &[String],
        capacity: usize,
    ) -> Result<Self, String> {
        child_process::Handle::spawn(command, args, capacity)
            .map(|handle| Self::new(Payload::ChildProcess(handle)))
    }
}
